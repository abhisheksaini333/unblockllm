//! NER engine: regex-based PII detection + optional ONNX NER via NER_MODEL_DIR.
//! When NER_MODEL_DIR is set and contains model.onnx + tokenizer.json, BERT-based
//! NER detects PERSON, ORGANIZATION, LOCATION entities. Regex always runs for
//! EMAIL, PHONE, SSN. Results are merged and deduplicated.
//! Runs inference in spawn_blocking to avoid blocking the async runtime.

use crate::masking::{merge_spans, spans_from_regex_only, Span};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::task;
use unblock_core::EntityType;

#[derive(Error, Debug)]
pub enum NerError {
    #[error("Inference error: {0}")]
    Inference(String),
    #[error("Model load error: {0}")]
    ModelLoad(String),
}

// BIO NER labels for dslim/bert-base-NER (protectai/bert-base-NER-onnx)
// Order from config.json id2label: 0=O, 1=B-MISC, 2=I-MISC, 3=B-PER, 4=I-PER, 5=B-ORG, 6=I-ORG, 7=B-LOC, 8=I-LOC
const NER_LABELS: &[&str] = &[
    "O", "B-MISC", "I-MISC", "B-PER", "I-PER", "B-ORG", "I-ORG", "B-LOC", "I-LOC",
];

fn ner_tag_to_entity_type(tag: &str) -> Option<EntityType> {
    match tag {
        "PER" => Some(EntityType::Person),
        "ORG" => Some(EntityType::Organization),
        "LOC" => Some(EntityType::Location),
        _ => None, // MISC doesn't map to a specific EntityType
    }
}

struct OnnxNer {
    session: Mutex<Session>,
    tokenizer: tokenizers::Tokenizer,
}

/// NER engine: regex (EMAIL, PHONE, SSN) + optional ONNX (PERSON, ORG, LOCATION).
pub struct NerEngine {
    onnx: Option<Arc<OnnxNer>>,
}

impl Clone for NerEngine {
    fn clone(&self) -> Self {
        Self {
            onnx: self.onnx.clone(),
        }
    }
}

impl NerEngine {
    /// Build engine from environment. Loads ONNX model if NER_MODEL_DIR is set
    /// and contains model.onnx + tokenizer.json. Falls back to regex-only gracefully.
    pub fn from_env() -> Result<Self, NerError> {
        let onnx = match std::env::var("NER_MODEL_DIR") {
            Ok(dir) if !dir.is_empty() => {
                let dir_path = std::path::Path::new(&dir);
                let model_path = dir_path.join("model.onnx");
                let tokenizer_path = dir_path.join("tokenizer.json");

                if !model_path.exists() {
                    tracing::warn!(
                        path = %model_path.display(),
                        "ONNX model not found, using regex-only NER"
                    );
                    None
                } else if !tokenizer_path.exists() {
                    tracing::warn!(
                        path = %tokenizer_path.display(),
                        "tokenizer.json not found, using regex-only NER"
                    );
                    None
                } else {
                    match Self::load_onnx(&model_path, &tokenizer_path) {
                        Ok(ner) => {
                            tracing::info!(
                                model = %model_path.display(),
                                "ONNX NER engine loaded"
                            );
                            Some(Arc::new(ner))
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "ONNX load failed, using regex-only NER");
                            None
                        }
                    }
                }
            }
            _ => {
                tracing::info!("NER_MODEL_DIR not set; using regex-only PII detection");
                None
            }
        };
        Ok(Self { onnx })
    }

    fn load_onnx(
        model_path: &std::path::Path,
        tokenizer_path: &std::path::Path,
    ) -> Result<OnnxNer, NerError> {
        let session = Session::builder()
            .map_err(|e| NerError::ModelLoad(format!("session builder: {}", e)))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| NerError::ModelLoad(format!("optimization level: {}", e)))?
            .with_intra_threads(1)
            .map_err(|e| NerError::ModelLoad(format!("intra threads: {}", e)))?
            .commit_from_file(model_path)
            .map_err(|e| NerError::ModelLoad(format!("load model: {}", e)))?;

        let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)
            .map_err(|e| NerError::ModelLoad(format!("load tokenizer: {}", e)))?;

        Ok(OnnxNer {
            session: Mutex::new(session),
            tokenizer,
        })
    }

    fn detect_sync(&self, text: &str) -> Vec<Span> {
        // Always run regex for EMAIL, PHONE, SSN
        let mut spans = spans_from_regex_only(text);

        // Run ONNX NER for PERSON, ORG, LOCATION if available
        if let Some(ref onnx) = self.onnx {
            match Self::detect_onnx(onnx, text) {
                Ok(onnx_spans) => {
                    tracing::debug!(count = onnx_spans.len(), "ONNX NER entities detected");
                    spans.extend(onnx_spans);
                }
                Err(e) => {
                    tracing::debug!(error = %e, "ONNX inference failed, regex-only for this request");
                }
            }
        }

        merge_spans(spans)
    }

    fn detect_onnx(onnx: &OnnxNer, text: &str) -> Result<Vec<Span>, NerError> {
        let encoding = onnx
            .tokenizer
            .encode(text, true)
            .map_err(|e| NerError::Inference(format!("tokenize: {}", e)))?;

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();
        let token_type_ids: Vec<i64> = encoding.get_type_ids().iter().map(|&t| t as i64).collect();
        let seq_len = input_ids.len();

        let shape = [1_usize, seq_len];
        let ids_tensor = ort::value::Tensor::from_array((&shape[..], input_ids))
            .map_err(|e| NerError::Inference(format!("input_ids tensor: {}", e)))?;
        let mask_tensor = ort::value::Tensor::from_array((&shape[..], attention_mask))
            .map_err(|e| NerError::Inference(format!("attention_mask tensor: {}", e)))?;
        let type_tensor = ort::value::Tensor::from_array((&shape[..], token_type_ids))
            .map_err(|e| NerError::Inference(format!("token_type_ids tensor: {}", e)))?;

        let mut session = onnx
            .session
            .lock()
            .map_err(|e| NerError::Inference(format!("session lock: {}", e)))?;
        let outputs = session
            .run(ort::inputs![
                "input_ids" => ids_tensor,
                "attention_mask" => mask_tensor,
                "token_type_ids" => type_tensor,
            ])
            .map_err(|e| NerError::Inference(format!("run: {}", e)))?;

        // Output shape: [1, seq_len, num_labels]
        let logits_tensor = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| NerError::Inference(format!("extract logits: {}", e)))?;
        let logits_shape = logits_tensor.0;
        let logits_data = logits_tensor.1;
        let num_labels_dim = if logits_shape.len() >= 3 {
            logits_shape[2] as usize
        } else {
            NER_LABELS.len()
        };

        let offsets = encoding.get_offsets();
        let special_tokens_mask = encoding.get_special_tokens_mask();

        // Decode BIO tags → entity spans
        let mut spans: Vec<Span> = Vec::new();
        // (char_start, char_end, entity_tag like "PER")
        let mut current: Option<(usize, usize, String)> = None;

        for i in 0..seq_len {
            // Skip special tokens ([CLS], [SEP], [PAD])
            if special_tokens_mask[i] == 1 {
                continue;
            }

            let (char_start, char_end) = offsets[i];
            if char_start == char_end {
                continue; // empty offset (shouldn't happen for real tokens)
            }

            // Argmax over label dimension (logits laid out as [batch=1, seq_len, num_labels])
            let num_labels = NER_LABELS.len().min(num_labels_dim);
            let row_offset = i * num_labels_dim; // batch 0, token i
            let mut best_idx = 0;
            let mut best_val = f32::NEG_INFINITY;
            for j in 0..num_labels {
                let val = logits_data[row_offset + j];
                if val > best_val {
                    best_val = val;
                    best_idx = j;
                }
            }

            let label = NER_LABELS.get(best_idx).copied().unwrap_or("O");

            if let Some(tag) = label.strip_prefix("B-") {
                // Finish previous entity
                if let Some((start, end, ref etype)) = current.take() {
                    if let Some(et) = ner_tag_to_entity_type(etype) {
                        spans.push(Span {
                            start,
                            end,
                            entity_type: et,
                        });
                    }
                }
                current = Some((char_start, char_end, tag.to_string()));
            } else if let Some(tag) = label.strip_prefix("I-") {
                if let Some(ref mut cur) = current {
                    if cur.2 == tag {
                        // Extend current entity
                        cur.1 = char_end;
                    } else {
                        // Type mismatch: finish current, start new
                        let (start, end, ref etype) = *cur;
                        if let Some(et) = ner_tag_to_entity_type(etype) {
                            spans.push(Span {
                                start,
                                end,
                                entity_type: et,
                            });
                        }
                        current = Some((char_start, char_end, tag.to_string()));
                    }
                } else {
                    // I- without B-: treat as B-
                    current = Some((char_start, char_end, tag.to_string()));
                }
            } else {
                // O label: finish current entity
                if let Some((start, end, ref etype)) = current.take() {
                    if let Some(et) = ner_tag_to_entity_type(etype) {
                        spans.push(Span {
                            start,
                            end,
                            entity_type: et,
                        });
                    }
                }
            }
        }

        // Finish trailing entity
        if let Some((start, end, ref etype)) = current {
            if let Some(et) = ner_tag_to_entity_type(etype) {
                spans.push(Span {
                    start,
                    end,
                    entity_type: et,
                });
            }
        }

        Ok(spans)
    }

    /// Returns true if the ONNX model is loaded (not regex-only fallback).
    pub fn has_onnx_model(&self) -> bool {
        self.onnx.is_some()
    }

    /// Run NER in a blocking task to avoid blocking the async runtime.
    pub async fn detect_async(&self, text: String) -> Result<Vec<Span>, NerError> {
        let engine = self.clone();
        task::spawn_blocking(move || Ok(engine.detect_sync(&text)))
            .await
            .map_err(|e| NerError::Inference(e.to_string()))?
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

    #[test]
    fn ner_label_mapping() {
        assert_eq!(ner_tag_to_entity_type("PER"), Some(EntityType::Person));
        assert_eq!(
            ner_tag_to_entity_type("ORG"),
            Some(EntityType::Organization)
        );
        assert_eq!(ner_tag_to_entity_type("LOC"), Some(EntityType::Location));
        assert_eq!(ner_tag_to_entity_type("MISC"), None);
        assert_eq!(ner_tag_to_entity_type("UNKNOWN"), None);
    }
}
