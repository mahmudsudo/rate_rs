use actix_web::{App, HttpServer, Responder, web};
use rate_rs::{InMemoryStore, RateLimitConfig, RateLimiter};
use std::sync::Arc;
use std::time::Duration;

async fn hello() -> impl Responder {
    "hello"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let store = InMemoryStore::new();
    let cfg = RateLimitConfig {
        capacity: 5,
        refill_tokens: 5,
        refill_interval: Duration::from_secs(60),
    };
    let limiter = Arc::new(RateLimiter::new(store, cfg));

    HttpServer::new(move || {
        App::new()
            .wrap(rate_rs::middleware::actix::ActixLimiter::new(
                limiter.clone(),
            ))
            .route("/", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
