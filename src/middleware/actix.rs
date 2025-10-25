use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{http, Error, HttpResponse};
use futures::future::{LocalBoxFuture, Ready, ok};
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::limiter::{RateLimitDecision, RateLimiter};

pub struct ActixLimiter<S: crate::limiter::StorageBackend> {
    limiter: Arc<RateLimiter<S>>,
}

impl<S: crate::limiter::StorageBackend> ActixLimiter<S> {
    pub fn new(limiter: Arc<RateLimiter<S>>) -> Self {
        Self { limiter }
    }
}

impl<S, B, St> Transform<St, ServiceRequest> for ActixLimiter<S>
where
    St: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S: crate::limiter::StorageBackend,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = ActixLimiterMiddleware<S, St>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: St) -> Self::Future {
        ok(ActixLimiterMiddleware {
            service: Rc::new(service),
            limiter: self.limiter.clone(),
        })
    }
}

pub struct ActixLimiterMiddleware<S: crate::limiter::StorageBackend, St> {
    service: Rc<St>,
    limiter: Arc<RateLimiter<S>>,
}

impl<S, St, B> Service<ServiceRequest> for ActixLimiterMiddleware<S, St>
where
    St: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S: crate::limiter::StorageBackend,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let limiter = self.limiter.clone();
        Box::pin(async move {
            let key = req
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("anon")
                .to_string();
            match limiter.check(&key).await {
                Ok(RateLimitDecision::Allowed { remaining }) => {
                    let mut res = srv.call(req).await?.map_into_boxed_body();
                    res.headers_mut().insert(
                        http::header::HeaderName::from_static("x-rate-limit-remaining"),
                        http::header::HeaderValue::from_str(&remaining.to_string()).unwrap(),
                    );
                    Ok(res)
                }
                Ok(RateLimitDecision::Limited { retry_after }) => {
                    let body = format!("rate limited; retry after {}s", retry_after.as_secs());
                    Ok(req.into_response(HttpResponse::TooManyRequests().body(body)).map_into_boxed_body())
                }
                Err(e) => Ok(req.into_response(
                    HttpResponse::InternalServerError().body(format!("limiter error: {}", e))
                ).map_into_boxed_body()),
            }
        })
    }
}
