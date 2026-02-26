# unblockllm Python package

SDK, CLI scanner, and ONNX NER benchmark.

## Install (editable)

```bash
pip install -e ".[dev]"
```

## Benchmark ONNX NER (<5ms target)

```bash
python -m unblockllm.benchmark.ner_benchmark
```

Output: `benchmark_report.json` and summary to stdout.

## CLI (Phase 4)

```bash
unblockllm-scan --text "Hello John Doe"
```
