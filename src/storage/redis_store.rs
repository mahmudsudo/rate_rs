use async_trait::async_trait;
use redis::AsyncCommands;
use serde_json;
use std::sync::Arc;

use crate::limiter::{RateLimitError, TokenState};
use crate::limiter::StorageBackend;

#[derive(Clone)]
pub struct RedisStore {
    client: Arc<redis::Client>,
}

impl RedisStore {
    pub fn new(redis_url: &str) -> Result<Self, RateLimitError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| RateLimitError::Storage(e.to_string()))?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    async fn get_conn(&self) -> Result<redis::aio::Connection, RateLimitError> {
        self.client
            .get_async_connection()
            .await
            .map_err(|e| RateLimitError::Storage(e.to_string()))
    }
}

#[async_trait]
impl StorageBackend for RedisStore {
    async fn get(&self, key: &str) -> Result<Option<TokenState>, RateLimitError> {
        let mut conn = self.get_conn().await?;
        let data: Option<String> = conn.get(key).await.map_err(|e| RateLimitError::Storage(e.to_string()))?;
        Ok(match data {
            Some(json) => serde_json::from_str(&json).ok(),
            None => None,
        })
    }

    async fn set(&self, key: &str, state: &TokenState) -> Result<(), RateLimitError> {
        let mut conn = self.get_conn().await?;
        let json = serde_json::to_string(state).unwrap();
        let ttl_secs = 3600; // optional: expire keys after 1 hour
        conn.set_ex::<_, _, ()>(key, json, ttl_secs)
            .await
            .map_err(|e| RateLimitError::Storage(e.to_string()))?;
        Ok(())
    }
}
