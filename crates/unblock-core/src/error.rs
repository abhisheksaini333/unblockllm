//! Error types for unblock-core. No PII in error messages.

use thiserror::Error;

/// Result type for core operations.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Core library errors. Messages must never contain PII.
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Redaction engine error: {0}")]
    RedactionEngine(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error (no PII): {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_error_display_no_pii() {
        let e = CoreError::InvalidInput("bad".to_string());
        let s = e.to_string();
        assert!(s.contains("Invalid input"));
        assert!(!s.to_lowercase().contains("pii"));
    }

    #[test]
    fn result_type() {
        let ok: Result<()> = Ok(());
        assert!(ok.is_ok());
        let err: Result<()> = Err(CoreError::Config("test".to_string()));
        assert!(err.is_err());
    }
}
