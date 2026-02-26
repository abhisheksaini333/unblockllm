//! Audit logging: metadata only, no PII. Writes to PostgreSQL.
//! When DATABASE_URL is unset, logging is no-op (for tests).

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

/// Create audit_logs table. Run once (e.g. migration or startup).
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id BIGSERIAL PRIMARY KEY,
            request_id TEXT NOT NULL,
            user_id TEXT,
            entity_count INT NOT NULL,
            entity_types TEXT NOT NULL DEFAULT '[]',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Write one audit entry. Never pass PII; only metadata.
pub async fn write(
    pool: &PgPool,
    request_id: &str,
    user_id: Option<&str>,
    entity_count: u32,
    entity_types: &[String],
) -> Result<(), sqlx::Error> {
    let types_json = serde_json::to_string(entity_types).unwrap_or_else(|_| "[]".to_string());
    sqlx::query(
        r#"
        INSERT INTO audit_logs (request_id, user_id, entity_count, entity_types)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(request_id)
    .bind(user_id)
    .bind(entity_count as i32)
    .bind(&types_json)
    .execute(pool)
    .await?;
    Ok(())
}

/// Optional audit logger. None when DATABASE_URL unset (no-op).
#[derive(Clone)]
pub struct AuditLog(Option<Arc<PgPool>>);

impl AuditLog {
    pub fn new(pool: Option<PgPool>) -> Self {
        Self(pool.map(Arc::new))
    }

    /// Log request metadata. No PII. entity_types = e.g. ["EMAIL", "PHONE"].
    pub async fn log(
        &self,
        request_id: &str,
        user_id: Option<&str>,
        entity_count: u32,
        entity_types: &[String],
    ) {
        if let Some(ref pool) = self.0 {
            if let Err(e) = write(pool.as_ref(), request_id, user_id, entity_count, entity_types).await
            {
                tracing::warn!(request_id = %request_id, error = %e, "audit write failed");
            }
        }
    }
}

/// Build PgPool from DATABASE_URL. Returns None if unset or invalid.
pub async fn pool_from_env() -> Result<Option<PgPool>, sqlx::Error> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => return Ok(None),
    };
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await?;
    migrate(&pool).await?;
    Ok(Some(pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn audit_log_no_pool_is_noop() {
        let log = AuditLog::new(None);
        log.log("req-1", None, 2, &["EMAIL".to_string(), "PHONE".to_string()]).await;
    }
}
