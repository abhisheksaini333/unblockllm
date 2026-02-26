//! NER (Named Entity Recognition) for PII detection.
//! Optional ONNX when NER_MODEL_DIR is set; otherwise regex-only.
//! Inference runs in spawn_blocking to avoid blocking the async runtime.

mod engine;

pub use engine::NerEngine;
