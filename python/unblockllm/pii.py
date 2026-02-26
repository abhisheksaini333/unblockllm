"""
Local PII detection via regex only. Used by CLI scanner.
No data leaves the machine; no network calls.
Patterns aligned with Phase 2 (Rust masking): EMAIL, PHONE, SSN.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import List, Tuple

# Entity type tags for report (no raw PII in logs)
EMAIL_TAG = "EMAIL"
PHONE_TAG = "PHONE"
SSN_TAG = "SSN"


@dataclass
class PIISpan:
    """A detected PII span: start, end, type. Value only in memory for report."""
    start: int
    end: int
    entity_type: str
    value: str  # Used only for report; never logged by default


def _regex_patterns() -> List[Tuple[str, re.Pattern[str]]]:
    """Regex patterns for EMAIL, PHONE, SSN. Same logic as Phase 2 Rust masking."""
    return [
        (EMAIL_TAG, re.compile(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")),
        (
            PHONE_TAG,
            re.compile(
                r"\b(?:\+1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}\b"
            ),
        ),
        (SSN_TAG, re.compile(r"\b[0-9]{3}-[0-9]{2}-[0-9]{4}\b")),
    ]


def find_pii_spans(text: str) -> List[PIISpan]:
    """
    Find all PII spans in text using regex only. No network; runs locally.
    Returns list of PIISpan (start, end, entity_type, value).
    """
    spans: List[PIISpan] = []
    for entity_type, pattern in _regex_patterns():
        for m in pattern.finditer(text):
            spans.append(
                PIISpan(
                    start=m.start(),
                    end=m.end(),
                    entity_type=entity_type,
                    value=m.group(),
                )
            )
    # Sort by start for stable report
    spans.sort(key=lambda s: (s.start, s.end))
    return spans
