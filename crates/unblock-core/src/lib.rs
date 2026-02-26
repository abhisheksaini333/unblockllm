//! Core library for unblockllm: types, errors, and redaction interfaces.
//!
//! PII must never leave local infrastructure unencrypted. This crate defines
//! the contracts used by the proxy and SDKs.

pub mod error;
pub mod types;

/// Generated Protobuf types for internal comms (redaction.proto).
#[allow(clippy::all)]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/unblockllm.redaction.v1.rs"));
}

pub use error::{CoreError, Result};
pub use types::{RedactionRequest, RedactionResult, EntitySpan, EntityType};
pub use types::MaskStrategy;
