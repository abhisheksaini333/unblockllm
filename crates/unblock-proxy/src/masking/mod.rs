//! PII masking: regex patterns + NER spans → placeholders and mapping.
//! No raw PII in logs; only counts and placeholder keys.

use regex::Regex;
use unblock_core::EntityType;

/// A span of text to mask (byte start, byte end, entity type).
#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub entity_type: EntityType,
}

/// Regex patterns for EMAIL, PHONE, SSN. Compiled once.
pub fn regex_patterns() -> Vec<(EntityType, Regex)> {
    let mut out = Vec::new();
    // Email: simple pattern (no PII in pattern itself)
    if let Ok(r) = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}") {
        out.push((EntityType::Email, r));
    }
    // US phone: 10 digits with optional separators
    if let Ok(r) = Regex::new(r#"\b(?:\+1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}\b"#) {
        out.push((EntityType::Phone, r));
    }
    // SSN: XXX-XX-XXXX
    if let Ok(r) = Regex::new(r"\b[0-9]{3}-[0-9]{2}-[0-9]{4}\b") {
        out.push((EntityType::Ssn, r));
    }
    out
}

/// Merge overlapping or adjacent spans; prefer earlier span on tie.
/// Spans must be sorted by start.
fn merge_spans(mut spans: Vec<Span>) -> Vec<Span> {
    spans.sort_by_key(|s| (s.start, s.end));
    let mut out: Vec<Span> = Vec::new();
    for s in spans {
        if let Some(last) = out.last_mut() {
            if s.start <= last.end {
                // overlap or adjacent: extend if needed
                if s.end > last.end {
                    last.end = s.end;
                }
                continue;
            }
        }
        out.push(s);
    }
    out
}

/// Apply masking: replace each span with placeholder [TYPE_N], build mapping.
/// Returns (masked_text, mapping: placeholder -> original).
/// Original values are not logged.
pub fn mask_text(text: &str, spans: &[Span]) -> (String, Vec<(String, String)>) {
    let mut mapping = Vec::new();
    let mut counters: std::collections::HashMap<EntityType, u32> = std::collections::HashMap::new();
    let type_tag = |t: &EntityType| -> String {
        match t {
            EntityType::Person => "PERSON",
            EntityType::Organization => "ORG",
            EntityType::Location => "LOC",
            EntityType::Email => "EMAIL",
            EntityType::Phone => "PHONE",
            EntityType::Ssn => "SSN",
            EntityType::Date => "DATE",
            EntityType::Custom(s) => s.as_str(),
        }
        .to_string()
    };

    let mut spans = spans.to_vec();
    spans.sort_by_key(|s| s.start);
    let spans = merge_spans(spans);

    let mut out = String::with_capacity(text.len());
    let mut last = 0_usize;
    for s in spans {
        if s.start < last {
            continue;
        }
        out.push_str(&text[last..s.start]);
        let n = *counters.entry(s.entity_type.clone()).or_insert(0) + 1;
        counters.insert(s.entity_type.clone(), n);
        let placeholder = format!("[{}_{}]", type_tag(&s.entity_type), n);
        let value = text[s.start..s.end].to_string();
        mapping.push((placeholder.clone(), value));
        out.push_str(&placeholder);
        last = s.end;
    }
    out.push_str(&text[last..]);
    (out, mapping)
}

/// Extract spans from regex patterns only (no NER). Used when NER is disabled or as fallback.
pub fn spans_from_regex_only(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    for (entity_type, re) in regex_patterns() {
        for mat in re.find_iter(text) {
            spans.push(Span {
                start: mat.start(),
                end: mat.end(),
                entity_type: entity_type.clone(),
            });
        }
    }
    spans
}

/// Re-identify: replace placeholders in response text with original values from mapping.
/// Used when streaming back to client. No PII in logs.
pub fn reidentify(text: &str, mapping: &[(String, String)]) -> String {
    let mut out = text.to_string();
    for (placeholder, value) in mapping {
        out = out.replace(placeholder, value);
    }
    out
}

/// Re-identify using a HashMap (from state store).
pub fn reidentify_map(text: &str, mapping: &std::collections::HashMap<String, String>) -> String {
    let mut out = text.to_string();
    for (placeholder, value) in mapping {
        out = out.replace(placeholder, value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_email_span() {
        let spans = spans_from_regex_only("Contact me at user@example.com please.");
        assert!(!spans.is_empty());
        assert_eq!(spans[0].entity_type, EntityType::Email);
        assert!(spans[0].start < spans[0].end);
    }

    #[test]
    fn mask_and_reidentify() {
        let text = "Email user@example.com and call 555-123-4567.";
        let spans = spans_from_regex_only(text);
        let (masked, map) = mask_text(text, &spans);
        assert!(masked.contains("[EMAIL_1]"));
        assert!(masked.contains("[PHONE_1]"));
        assert!(!masked.contains("user@example.com"));
        let back = reidentify(&masked, &map);
        assert!(back.contains("user@example.com"));
        assert!(back.contains("555-123-4567"));
    }
}
