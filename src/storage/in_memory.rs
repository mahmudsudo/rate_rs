use crate::limiter::{RateLimitError, TokenState};
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct InMemoryStore {
    map: Arc<DashMap<String, TokenState>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self {
            map: Arc::new(DashMap::new()),
        }
    }
}



#[async_trait::async_trait]
impl crate::limiter::StorageBackend for InMemoryStore {
    async fn get(&self, key: &str) -> Result<Option<TokenState>, RateLimitError> {
        Ok(self
            .map
            .get(key)
            .map(|v| v.value().clone()))
    }

    async fn set(&self, key: &str, state: &TokenState) -> Result<(), RateLimitError> {
        self.map.insert(key.to_string(), state.clone());
        Ok(())
    }
}
