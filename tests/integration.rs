#[tokio::test]
async fn token_bucket_basic() {
use rate_rs::{RateLimiter, RateLimitConfig, InMemoryStore};
use std::time::Duration;

// InMemoryStore already provides the required StorageBackend implementation.
let store = InMemoryStore::new();
let cfg = RateLimitConfig { capacity: 3, refill_tokens: 1, refill_interval: Duration::from_secs(1) };
let limiter = RateLimiter::new(store, cfg);


// use same key 3 times -> allowed, 4th -> limited
assert!(matches!(limiter.check("u1").await.unwrap(), rate_rs::RateLimitDecision::Allowed { .. }));
assert!(matches!(limiter.check("u1").await.unwrap(), rate_rs::RateLimitDecision::Allowed { .. }));
assert!(matches!(limiter.check("u1").await.unwrap(), rate_rs::RateLimitDecision::Allowed { .. }));
assert!(matches!(limiter.check("u1").await.unwrap(), rate_rs::RateLimitDecision::Limited { .. }));


// wait and refill
tokio::time::sleep(Duration::from_secs(1)).await;
assert!(matches!(limiter.check("u1").await.unwrap(), rate_rs::RateLimitDecision::Allowed { .. }));
}