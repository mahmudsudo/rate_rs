use std::sync::Arc;
use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use crate::limiter::{RateLimiter, RateLimitDecision, StorageBackend};
use axum::extract::State;
/// Framework-agnostic Axum middleware for rate limiting.
/// Use it with `Router::layer(axum::middleware::from_fn_with_state(...))`.
pub async fn rate_limit_middleware<S>(
    State(limiter): State<Arc<RateLimiter<S>>>,
     req: Request,
    next: Next,
) -> impl IntoResponse
where
    S: StorageBackend,
{
    // Extract key (can be IP, header, etc.)
    let key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anon");

    match limiter.check(key).await {
        Ok(RateLimitDecision::Allowed { remaining }) => {
            let mut response = next.run(req).await;
            response.headers_mut().insert(
                "x-rate-limit-remaining",
                HeaderValue::from_str(&remaining.to_string()).unwrap(),
            );
            response
        }
        Ok(RateLimitDecision::Limited { retry_after }) => {
            (
                StatusCode::TOO_MANY_REQUESTS,
                format!("Rate limited; retry after {}s", retry_after.as_secs()),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Rate limiter error: {}", e),
        )
            .into_response(),
    }
}


