"""Tests for benchmark module. No PII in test data."""

import pytest


def test_benchmark_imports() -> None:
    """Ensure benchmark module can be imported."""
    from unblockllm.benchmark import ner_benchmark

    assert ner_benchmark.BENCHMARK_SAMPLES
    assert len(ner_benchmark.BENCHMARK_SAMPLES) >= 1
    # No PII in samples
    for s in ner_benchmark.BENCHMARK_SAMPLES:
        assert isinstance(s, str)
        assert len(s) > 0
