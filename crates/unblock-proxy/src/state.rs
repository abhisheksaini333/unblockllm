//! Mapping store for re-identification. Phase 3: Redis-backed with TTL.
//! When REDIS_URL is unset, falls back to in-memory store for tests.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const REDIS_KEY_PREFIX: &str = "unblock:map:";
const MAPPING_TTL_SECS: u64 = 60;

/// Placeholder -> original value. No raw PII in logs.
pub type Mapping = HashMap<String, String>;

/// Store backend: Redis (when REDIS_URL set) or in-memory (fallback).
#[derive(Clone)]
pub enum Store {
    Redis(RedisStore),
    Memory(Arc<RwLock<HashMap<String, Mapping>>>),
}

pub struct RedisStore {
    conn: Arc<tokio::sync::Mutex<redis::aio::ConnectionManager>>,
}

impl Clone for RedisStore {
    fn clone(&self) -> Self {
        Self { conn: Arc::clone(&self.conn) }
    }
}

impl Store {
    /// Build from env: REDIS_URL => Redis; else in-memory (tests/local).
    pub async fn from_env() -> Result<Self, StateError> {
        let url = match std::env::var("REDIS_URL") {
            Ok(u) if !u.is_empty() => u,
            _ => {
                tracing::debug!("REDIS_URL unset; using in-memory store");
                return Ok(Store::Memory(Arc::new(RwLock::new(HashMap::new()))));
            }
        };
        let client = redis::Client::open(url).map_err(|e| StateError::Redis(e.to_string()))?;
        let conn = client
            .get_connection_manager()
            .await
            .map_err(|e| StateError::Redis(e.to_string()))?;
        Ok(Store::Redis(RedisStore {
            conn: Arc::new(tokio::sync::Mutex::new(conn)),
        }))
    }

    /// In-memory only (for tests).
    pub fn new_memory() -> Self {
        Store::Memory(Arc::new(RwLock::new(HashMap::new())))
    }

    /// Generate a new request id. For Memory backend, reserves an empty mapping.
    /// Redis: no-op for id generation; mapping is created on first insert.
    pub async fn new_request(&self) -> Result<String, StateError> {
        let id = Uuid::new_v4().to_string();
        if let Store::Memory(m) = self {
            m.write().await.insert(id.clone(), Mapping::new());
        }
        Ok(id)
    }

    pub async fn insert(
        &self,
        request_id: &str,
        placeholder: String,
        value: String,
    ) -> Result<(), StateError> {
        match self {
            Store::Redis(r) => {
                let key = format!("{}{}", REDIS_KEY_PREFIX, request_id);
                let mut conn = r.conn.lock().await;
                redis::cmd("HSET")
                    .arg(&key)
                    .arg(&placeholder)
                    .arg(&value)
                    .query_async::<()>(&mut *conn)
                    .await
                    .map_err(|e| StateError::Redis(e.to_string()))?;
                redis::cmd("EXPIRE")
                    .arg(&key)
                    .arg(MAPPING_TTL_SECS)
                    .query_async::<()>(&mut *conn)
                    .await
                    .map_err(|e| StateError::Redis(e.to_string()))?;
            }
            Store::Memory(m) => {
                if let Some(map) = m.write().await.get_mut(request_id) {
                    map.insert(placeholder, value);
                } else {
                    let mut map = Mapping::new();
                    map.insert(placeholder, value);
                    m.write().await.insert(request_id.to_string(), map);
                }
            }
        }
        Ok(())
    }

    pub async fn get_mapping(&self, request_id: &str) -> Result<Option<Mapping>, StateError> {
        match self {
            Store::Redis(r) => {
                let key = format!("{}{}", REDIS_KEY_PREFIX, request_id);
                let mut conn = r.conn.lock().await;
                let raw: Result<HashMap<String, String>, redis::RedisError> =
                    redis::cmd("HGETALL").arg(&key).query_async(&mut *conn).await;
                Ok(Some(raw.map_err(|e| StateError::Redis(e.to_string()))?))
            }
            Store::Memory(m) => Ok(m.read().await.get(request_id).cloned()),
        }
    }

    pub async fn remove(&self, request_id: &str) -> Result<(), StateError> {
        match self {
            Store::Redis(r) => {
                let key = format!("{}{}", REDIS_KEY_PREFIX, request_id);
                let mut conn = r.conn.lock().await;
                redis::cmd("DEL")
                    .arg(&key)
                    .query_async::<()>(&mut *conn)
                    .await
                    .map_err(|e| StateError::Redis(e.to_string()))?;
            }
            Store::Memory(m) => {
                m.write().await.remove(request_id);
            }
        }
        Ok(())
    }

    pub async fn len(&self) -> usize {
        match self {
            Store::Redis(r) => {
                let mut conn = r.conn.lock().await;
                let keys: Result<Vec<String>, redis::RedisError> =
                    redis::cmd("KEYS").arg(format!("{}*", REDIS_KEY_PREFIX)).query_async(&mut *conn).await;
                keys.map(|k| k.len()).unwrap_or(0)
            }
            Store::Memory(m) => m.read().await.len(),
        }
    }

    /// Returns true if the store has no mappings.
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Redis error: {0}")]
    Redis(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_store_insert_get_remove() {
        let store = Store::new_memory();
        let id = store.new_request().await.expect("new_request");
        store
            .insert(&id, "[EMAIL_1]".to_string(), "u@x.com".to_string())
            .await
            .expect("insert");
        let m = store.get_mapping(&id).await.expect("get_mapping").expect("mapping");
        assert_eq!(m.get("[EMAIL_1]").map(String::as_str), Some("u@x.com"));
        store.remove(&id).await.expect("remove");
        assert!(store.get_mapping(&id).await.expect("get_mapping").is_none());
    }
}
