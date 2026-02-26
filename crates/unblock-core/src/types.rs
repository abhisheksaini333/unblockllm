//! Core types for redaction and re-identification. No PII in logs or serialization of raw text.

use serde::{Deserialize, Serialize};

/// Strategy for masking detected PII.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskStrategy {
    /// Replace with fixed placeholder (e.g. [PERSON_1]).
    Placeholder,
    /// Replace with type-only tag (e.g. [PER]).
    TypeTag,
    /// Replace with hash of value (for deterministic re-id).
    Hash,
}

/// Entity type from NER or regex.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Email,
    Phone,
    Ssn,
    Date,
    Custom(String),
}

/// A span of text identified as PII.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySpan {
    pub start: usize,
    pub end: usize,
    pub entity_type: EntityType,
    /// Optional score from NER (0.0–1.0). Not logged in production.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

/// Request to redact PII from text. Raw text must not be logged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRequest {
    /// Opaque request id for audit (no PII).
    pub request_id: String,
    /// Mask strategy to apply.
    pub strategy: MaskStrategy,
    /// Optional list of entity types to redact (empty = all).
    #[serde(default)]
    pub entity_types: Vec<EntityType>,
}

/// Result of redaction: masked text + mapping for re-identification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionResult {
    /// Masked text (safe to send upstream).
    pub masked_text: String,
    /// Token/entity mapping for re-identification (stored ephemerally).
    pub mapping: Vec<EntitySpan>,
    /// Count of entities redacted (for audit; no raw PII).
    pub entity_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_strategy_serialize_roundtrip() {
        for strategy in [
            MaskStrategy::Placeholder,
            MaskStrategy::TypeTag,
            MaskStrategy::Hash,
        ] {
            let j = serde_json::to_string(&strategy).expect("serialize");
            let d: MaskStrategy = serde_json::from_str(&j).expect("deserialize");
            assert_eq!(strategy, d);
        }
    }

    #[test]
    fn entity_span_roundtrip() {
        let span = EntitySpan {
            start: 0,
            end: 5,
            entity_type: EntityType::Person,
            score: Some(0.99),
        };
        let j = serde_json::to_string(&span).expect("serialize");
        let d: EntitySpan = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(span.start, d.start);
        assert_eq!(span.end, d.end);
        assert_eq!(d.score, Some(0.99));
    }

    #[test]
    fn redaction_request_roundtrip() {
        let req = RedactionRequest {
            request_id: "req-1".to_string(),
            strategy: MaskStrategy::Hash,
            entity_types: vec![EntityType::Email, EntityType::Phone],
        };
        let j = serde_json::to_string(&req).expect("serialize");
        let d: RedactionRequest = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(d.request_id, "req-1");
        assert_eq!(d.entity_types.len(), 2);
    }

    #[test]
    fn redaction_result_entity_count() {
        let res = RedactionResult {
            masked_text: "Hello [PER_1]".to_string(),
            mapping: vec![EntitySpan {
                start: 6,
                end: 14,
                entity_type: EntityType::Person,
                score: None,
            }],
            entity_count: 1,
        };
        assert_eq!(res.entity_count, 1);
        assert_eq!(res.mapping.len(), 1);
    }
}
