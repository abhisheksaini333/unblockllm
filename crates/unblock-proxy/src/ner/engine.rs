//! NER engine: regex-based PII detection. Optional ONNX via NER_MODEL_DIR (see docs).
//! Runs inference in spawn_blocking to avoid blocking the async runtime.

use crate::masking::{Span, spans_from_regex_only};
use thiserror::Error;
use tokio::task;

#[derive(Error, Debug)]
pub enum NerError {
    #[error("Inference error: {0}")]
    Inference(String),
}

/// NER engine. Phase 2: regex-only (EMAIL, PHONE, SSN). Add ONNX when NER_MODEL_DIR + model.onnx available.
pub struct NerEngine;

impl NerEngine {
    /// Build engine. Uses regex-only unless NER_MODEL_DIR points to model.onnx (future ONNX support).
    pub fn from_env() -> Result<Self, NerError> {
        if std::env::var("NER_MODEL_DIR").is_ok() {
            tracing::debug!("NER_MODEL_DIR set; ONNX support can be enabled with ort feature");
        }
        Ok(Self)
    }

    fn detect_sync(&self, text: &str) -> Vec<Span> {
        spans_from_regex_only(text)
    }

    /// Run NER in a blocking task to avoid blocking the async runtime.
    pub async fn detect_async(&self, text: String) -> Result<Vec<Span>, NerError> {
        let engine = Self;
        task::spawn_blocking(move || Ok(engine.detect_sync(&text)))
            .await
            .map_err(|e| NerError::Inference(e.to_string()))?
    }
}

impl Clone for NerEngine {
    fn clone(&self) -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn engine_regex_only() {
        let engine = NerEngine::from_env().expect("build");
        let text = "Contact user@example.com and 555-123-4567.";
        let spans = engine.detect_async(text.to_string()).await.expect("detect");
        assert!(!spans.is_empty());
    }
}
