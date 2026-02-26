"""
ONNX NER model benchmark: verify inference latency <5ms per request.

Model: protectai/bert-base-NER-onnx (ONNX-exported dslim/bert-base-NER).
HuggingFace: https://huggingface.co/protectai/bert-base-NER-onnx

Falls back to dslim/bert-base-NER with Optimum ONNX export if needed.
Output: benchmark_report.json + stdout summary. No PII in logs or report.
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
import time
from pathlib import Path

# Optional: use optimum for loading ONNX from HuggingFace
try:
    from optimum.onnxruntime import ORTModelForTokenClassification
    from transformers import AutoTokenizer, pipeline

    HAS_OPTIMUM = True
except ImportError:
    HAS_OPTIMUM = False

# Fallback: onnxruntime only with local .onnx (e.g. downloaded from HF)
try:
    import onnxruntime as ort
    import numpy as np

    HAS_ONNX = True
except ImportError:
    HAS_ONNX = False


# Default model: pre-exported ONNX for NER (CoNLL-2003: PER, ORG, LOC, MISC)
# HuggingFace: https://huggingface.co/protectai/bert-base-NER-onnx
DEFAULT_MODEL_ID = "protectai/bert-base-NER-onnx"
# Alternative: dslim/bert-base-NER (Optimum loads and runs via ONNX Runtime)
FALLBACK_MODEL_ID = "dslim/bert-base-NER"

# Benchmark sample (no PII; generic sentences)
BENCHMARK_SAMPLES = [
    "The company was founded in New York by John Smith.",
    "Dr. Jane Doe works at the University of California.",
    "Please send the report to the office in London by Monday.",
]


def load_ort_pipeline(model_id: str):
    """Load token classification pipeline with ONNX Runtime backend."""
    if not HAS_OPTIMUM:
        raise RuntimeError(
            "Install optimum and transformers: pip install optimum[onnxruntime] transformers"
        )
    tokenizer = AutoTokenizer.from_pretrained(model_id)
    model = ORTModelForTokenClassification.from_pretrained(model_id)
    pipe = pipeline(
        "ner",
        model=model,
        tokenizer=tokenizer,
        aggregation_strategy="simple",
    )
    return pipe


def run_inference_warmup(pipe, samples: list[str], warmup_runs: int = 5) -> None:
    """Warmup to stabilize latency (no timing)."""
    for _ in range(warmup_runs):
        for text in samples:
            pipe(text)


def benchmark_latency(
    pipe, samples: list[str], num_runs: int = 100
) -> tuple[list[float], list[float]]:
    """Measure per-request latency in ms. Returns (per_run_ms, per_sample_avg_ms)."""
    per_run_ms: list[float] = []
    for _ in range(num_runs):
        for text in samples:
            start = time.perf_counter()
            pipe(text)
            elapsed_ms = (time.perf_counter() - start) * 1000
            per_run_ms.append(elapsed_ms)
    per_sample_avg = [
        statistics.mean(per_run_ms[i :: len(samples)]) for i in range(len(samples))
    ]
    return per_run_ms, per_sample_avg


def main() -> int:
    parser = argparse.ArgumentParser(description="ONNX NER benchmark (<5ms target)")
    parser.add_argument(
        "--model",
        default=DEFAULT_MODEL_ID,
        help=f"Model ID (default: {DEFAULT_MODEL_ID})",
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=100,
        help="Number of inference runs per sample (default: 100)",
    )
    parser.add_argument(
        "--warmup",
        type=int,
        default=5,
        help="Warmup runs (default: 5)",
    )
    parser.add_argument(
        "--output",
        default="benchmark_report.json",
        help="Output JSON report path (default: benchmark_report.json)",
    )
    args = parser.parse_args()

    if not HAS_OPTIMUM:
        print(
            "ERROR: Install optimum and transformers: pip install optimum[onnxruntime] transformers",
            file=sys.stderr,
        )
        return 1

    print(f"Loading model: {args.model}")
    try:
        pipe = load_ort_pipeline(args.model)
    except Exception as e:
        # Try fallback if protectai model not found
        if args.model == DEFAULT_MODEL_ID:
            print(f"Trying fallback: {FALLBACK_MODEL_ID}")
            try:
                pipe = load_ort_pipeline(FALLBACK_MODEL_ID)
                args.model = FALLBACK_MODEL_ID
            except Exception as e2:
                print(f"ERROR: {e2}", file=sys.stderr)
                return 1
        else:
            print(f"ERROR: {e}", file=sys.stderr)
            return 1

    print("Warmup...")
    run_inference_warmup(pipe, BENCHMARK_SAMPLES, warmup_runs=args.warmup)

    print(f"Benchmark: {args.runs} runs x {len(BENCHMARK_SAMPLES)} samples")
    per_run_ms, per_sample_avg = benchmark_latency(pipe, BENCHMARK_SAMPLES, num_runs=args.runs)

    all_ms = per_run_ms
    mean_ms = statistics.mean(all_ms)
    median_ms = statistics.median(all_ms)
    p95_ms = statistics.quantiles(all_ms, n=20)[18] if len(all_ms) >= 20 else max(all_ms)
    p99_ms = statistics.quantiles(all_ms, n=100)[98] if len(all_ms) >= 100 else max(all_ms)
    stdev_ms = statistics.stdev(all_ms) if len(all_ms) > 1 else 0.0

    target_ms = 5.0
    passed = mean_ms < target_ms

    report = {
        "model_id": args.model,
        "target_latency_ms": target_ms,
        "passed": passed,
        "runs": args.runs,
        "samples_count": len(BENCHMARK_SAMPLES),
        "latency_ms": {
            "mean": round(mean_ms, 4),
            "median": round(median_ms, 4),
            "p95": round(p95_ms, 4),
            "p99": round(p99_ms, 4),
            "stdev": round(stdev_ms, 4),
        },
        "per_sample_avg_ms": [round(x, 4) for x in per_sample_avg],
    }

    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(report, f, indent=2)

    print("\n--- ONNX NER Benchmark Report ---")
    print(f"Model: {args.model}")
    print(f"Target: <{target_ms} ms")
    print(f"Mean:   {mean_ms:.2f} ms")
    print(f"Median: {median_ms:.2f} ms")
    print(f"P95:    {p95_ms:.2f} ms")
    print(f"P99:    {p99_ms:.2f} ms")
    print(f"Result: {'PASS' if passed else 'FAIL'} (<5ms)")
    print(f"Report: {out_path.absolute()}")

    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
