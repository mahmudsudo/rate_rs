use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use thiserror::Error;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub capacity: u32,
    pub refill_tokens: u32,
    pub refill_interval: Duration,
}

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Debug, Clone)]
pub enum RateLimitDecision {
    Allowed { remaining: u32 },
    Limited { retry_after: Duration },
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct TokenState {
    pub tokens: u32,
    pub last_refill: DateTime<Utc>,
}

#[async_trait]
pub trait StorageBackend: Send + Sync + 'static {
    async fn get(&self, key: &str) -> Result<Option<TokenState>, RateLimitError>;
    async fn set(&self, key: &str, state: &TokenState) -> Result<(), RateLimitError>;
}

#[derive(Debug, Clone)]
pub struct RateLimiter<S: StorageBackend> {
    store: Arc<S>,
    config: RateLimitConfig,
}

impl<S: StorageBackend> RateLimiter<S> {
    pub fn new(store: S, config: RateLimitConfig) -> Self {
        Self {
            store: Arc::new(store),
            config,
        }
    }

    pub async fn check(&self, key: &str) -> Result<RateLimitDecision, RateLimitError> {
        let now = Utc::now();
        let mut state = self.store.get(key).await?;

        let mut token_state = match state.take() {
            Some(mut s) => {
                // refill
                let elapsed = now
                    .signed_duration_since(s.last_refill)
                    .to_std()
                    .unwrap_or(Duration::ZERO);
                let intervals = elapsed.as_secs_f64() / self.config.refill_interval.as_secs_f64();
                if intervals >= 1.0 {
                    let add = (intervals.floor() as u32).saturating_mul(self.config.refill_tokens);
                    s.tokens = (s.tokens + add).min(self.config.capacity);
                    s.last_refill = now;
                }
                s
            }
            None => TokenState {
                tokens: self.config.capacity,
                last_refill: now,
            },
        };

        if token_state.tokens > 0 {
            token_state.tokens -= 1;
            self.store.set(key, &token_state).await?;
            Ok(RateLimitDecision::Allowed {
                remaining: token_state.tokens,
            })
        } else {
            // compute retry after
            let retry_after = self.config.refill_interval;
            Ok(RateLimitDecision::Limited { retry_after })
        }
    }
}
