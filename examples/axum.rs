use axum::{
    middleware,
    routing::get,
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use rate_rs::{RateLimiter, RateLimitConfig, InMemoryStore};
use rate_rs::middleware::axum::rate_limit_middleware;

#[tokio::main]
async fn main() {
    // Set up in-memory limiter
    let store = InMemoryStore::new();
    let cfg = RateLimitConfig {
        capacity: 5,
        refill_tokens: 5,
        refill_interval: Duration::from_secs(60),
    };
    let limiter = Arc::new(RateLimiter::new(store, cfg));

    // Build router
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        // attach middleware with shared limiter
        .layer(middleware::from_fn_with_state(limiter.clone(), rate_limit_middleware))
        .with_state(limiter);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("Listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
