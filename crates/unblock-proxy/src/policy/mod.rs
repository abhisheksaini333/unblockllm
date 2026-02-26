//! Policy engine: YAML config for mask/block lists. Hot-reload via file watch or remote poll.

use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;

/// Policy config (from policy.yaml). No PII.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PolicyConfig {
    #[serde(default)]
    pub mask: Vec<String>,
    #[serde(default)]
    pub block: Vec<String>,
}

/// Live policy with optional hot-reload.
#[derive(Clone)]
pub struct PolicyEngine(Arc<RwLock<PolicyConfig>>);

impl PolicyEngine {
    /// Load from path. Returns default (allow all mask, block none) on error.
    pub fn load(path: &Path) -> Self {
        let config = Self::load_config(path);
        Self(Arc::new(RwLock::new(config)))
    }

    fn load_config(path: &Path) -> PolicyConfig {
        let data = match std::fs::read_to_string(path) {
            Ok(d) => d,
            Err(_) => return PolicyConfig::default(),
        };
        serde_yaml::from_str(&data).unwrap_or_default()
    }

    /// Current config (copy).
    pub async fn get(&self) -> PolicyConfig {
        self.0.read().await.clone()
    }

    /// Set config from remote (e.g. dashboard). Used when DASHBOARD_URL is set.
    pub async fn set_config(&self, config: PolicyConfig) {
        *self.0.write().await = config;
    }

    /// Whether to mask this entity type (policy says mask, and not in block).
    pub async fn should_mask(&self, entity_tag: &str) -> bool {
        let config = self.0.read().await;
        if config.block.iter().any(|s| s.eq_ignore_ascii_case(entity_tag)) {
            return false; // blocked type: caller should reject request
        }
        if config.mask.is_empty() {
            return true; // mask all
        }
        config.mask.iter().any(|s| s.eq_ignore_ascii_case(entity_tag))
    }

    /// Whether this entity type is in the block list (request should be rejected).
    pub async fn is_blocked(&self, entity_tag: &str) -> bool {
        let config = self.0.read().await;
        config.block.iter().any(|s| s.eq_ignore_ascii_case(entity_tag))
    }

    /// Spawn a task that reloads policy from path every `interval_secs`.
    pub fn spawn_reload(self, path: std::path::PathBuf, interval_secs: u64) {
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            loop {
                ticker.tick().await;
                let config = Self::load_config(&path);
                *self.0.write().await = config;
                tracing::debug!(path = %path.display(), "policy reloaded");
            }
        });
    }
}

/// JSON response from dashboard /api/v1/policy. No PII.
#[derive(Debug, Deserialize)]
struct RemotePolicyResponse {
    #[serde(default)]
    mask: Vec<String>,
    #[serde(default)]
    block: Vec<String>,
}

/// Spawn a task that polls dashboard for policy. Does not block; runs in background.
pub fn spawn_remote_poll(policy: PolicyEngine, dashboard_url: String, proxy_api_key: String) {
    let base = dashboard_url.trim_end_matches('/').to_string();
    let url = format!("{}/api/v1/policy", base);
    tokio::spawn(async move {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "policy remote client build failed");
                return;
            }
        };
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            let req = client
                .get(&url)
                .header("X-Proxy-API-Key", &proxy_api_key);
            match req.send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<RemotePolicyResponse>().await {
                        Ok(remote) => {
                            let config = PolicyConfig {
                                mask: remote.mask,
                                block: remote.block,
                            };
                            policy.set_config(config).await;
                            tracing::debug!("policy updated from remote");
                        }
                        Err(e) => tracing::debug!(error = %e, "policy remote parse failed"),
                    }
                }
                Ok(resp) => tracing::debug!(status = %resp.status(), "policy remote fetch non-ok"),
                Err(e) => tracing::debug!(error = %e, "policy remote fetch failed"),
            }
        }
    });
}

fn entity_type_to_tag(t: &unblock_core::EntityType) -> &'static str {
    use unblock_core::EntityType;
    match t {
        EntityType::Person => "PERSON",
        EntityType::Organization => "ORGANIZATION",
        EntityType::Location => "LOCATION",
        EntityType::Email => "EMAIL",
        EntityType::Phone => "PHONE",
        EntityType::Ssn => "SSN",
        EntityType::Date => "DATE",
        EntityType::Custom(_) => "CUSTOM",
    }
}

/// Filter spans by policy: only those that should be masked; check for blocked.
pub async fn filter_spans_by_policy(
    policy: &PolicyEngine,
    spans: &[crate::masking::Span],
) -> Result<Vec<crate::masking::Span>, PolicyBlockError> {
    for s in spans {
        let tag = entity_type_to_tag(&s.entity_type);
        if policy.is_blocked(tag).await {
            return Err(PolicyBlockError::Blocked(tag.to_string()));
        }
    }
    let mut out = Vec::new();
    for s in spans {
        let tag = entity_type_to_tag(&s.entity_type);
        if policy.should_mask(tag).await {
            out.push(s.clone());
        }
    }
    Ok(out)
}

#[derive(Debug, thiserror::Error)]
pub enum PolicyBlockError {
    #[error("Request blocked by policy: entity type {0} is not allowed")]
    Blocked(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn policy_default_masks_all() {
        let engine = PolicyEngine::load(Path::new("/nonexistent"));
        assert!(engine.should_mask("EMAIL").await);
        assert!(!engine.is_blocked("EMAIL").await);
    }
}
