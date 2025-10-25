# rate_rs — Universal Rate Limiting Library for Rust Web Frameworks

**rate_rs** is a framework-agnostic rate limiting crate for Rust.  
It provides an easy-to-use, configurable API that integrates seamlessly with **Axum**, **Actix Web**, and other async web frameworks.

Built with performance, extensibility, and clarity in mind, `rate_rs` helps you protect your APIs, control throughput, and prevent abuse, whether running in-memory or with a distributed store (Redis backend in development).

---

## ✨ Features

- ⚙️ Framework-agnostic core usable with any async runtime  
- 🧩 Native Axum and Actix Web middleware  
- ⏱ Token Bucket algorithm for predictable rate limiting  
- 🔧 Configurable rate and refill interval  
- 💾 Pluggable storage backends  
  - In-memory (default)  
  - Redis (optional, work in progress)  
- 🧪 Fully unit- and integration-tested  
- 💯 100% safe Rust, async-first (tokio-based)

---

## 🚀 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rate_rs = { git = "https://github.com/mahmudsudo/rate_rs" }
```

## Example: Axum

```rust
use axum::{routing::get, Router};
use rate_rs::{limiter::RateLimiter, middleware::axum::RateLimitLayer, storage::in_memory::InMemoryStore};
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    let store = Arc::new(InMemoryStore::new());
    let limiter = Arc::new(RateLimiter::new(store, 5, Duration::from_secs(10)));
    let layer = ServiceBuilder::new().layer(RateLimitLayer::new(limiter));

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .layer(layer);

    println!("Server running on http://127.0.0.1:3000");
    axum::serve(tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap(), app)
        .await
        .unwrap();
}
```


## Example: Actix Web

```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use rate_rs::{limiter::RateLimiter, middleware::actix::RateLimit, storage::in_memory::InMemoryStore};
use std::{sync::Arc, time::Duration};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let store = Arc::new(InMemoryStore::new());
    let limiter = Arc::new(RateLimiter::new(store, 5, Duration::from_secs(10)));

    println!("Server running on http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .wrap(RateLimit::new(limiter.clone()))
            .route("/", web::get().to(|| async { HttpResponse::Ok().body("Hello from Actix!") }))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```
## Configuration
| Parameter | Description                             | Example                         |
| --------- | --------------------------------------- | ------------------------------- |
| `limit`   | Maximum allowed requests per window     | `5`                             |
| `window`  | Duration before reset                   | `Duration::from_secs(10)`       |
| `storage` | Backend store implementation            | `InMemoryStore` or `RedisStore` |
| `key`     | Unique identifier per client (optional) | `"user:123"`                    |


