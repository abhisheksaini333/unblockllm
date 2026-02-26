# Phase 2 Prerequisites — Confirmed Choices

Decisions and verification results before starting **Phase 2: Core Proxy Development**.

---

## 1. Benchmark verification

**Question:** Did the NER benchmark confirm &lt;5ms inference on target hardware?

**Result (run on target hardware):**

| Metric | Value | Target |
|--------|--------|--------|
| **Mean** | **14.28 ms** | &lt;5 ms |
| **Median** | 11.73 ms | — |
| **P95** | **25.47 ms** | — |
| **P99** | 86.48 ms | — |
| **Outcome** | **FAIL** vs 5ms | — |

- **Model:** `protectai/bert-base-NER-onnx`
- **Run:** `cd python && python -m unblockllm.benchmark.ner_benchmark --runs 50 --output benchmark_report.json`
- **Environment:** MPS (Apple Silicon) observed in run; exact hardware = your target dev machine.

**Implication:** Total proxy overhead (mask + unmask) may exceed 20ms with current model. Proceed with Phase 2 as planned; address latency in Phase 2 or Phase 6 via:

- Quantized ONNX (INT8), or  
- Smaller NER model (e.g. DistilBERT NER), or  
- Optional async/batch NER where acceptable.

---

## 2. Infrastructure choice (Phase 3)

**Question:** Local Docker (Redis/Postgres) for development or Managed Cloud (e.g. AWS ElastiCache/RDS) from the start?

**Decision:** **Local Docker for development.**

- Use **Redis** and **PostgreSQL** in Docker for local/dev.
- Implement **cloud-agnostic** configuration: all connection strings and feature toggles via **environment variables** (e.g. `REDIS_URL`, `DATABASE_URL`), so we can switch to ElastiCache/RDS (or other managed services) later without code changes.
- Recommendation: Start with Local Docker for speed; keep code env-driven for future cloud.

---

## 3. Priority endpoints

**Question:** Support every LLM API immediately, or prioritize one?

**Decision:** **Prioritize OpenAI Chat Completions first.**

- **Endpoint:** `POST /v1/chat/completions`
- **Rationale:** Covers the majority of use cases; other endpoints (e.g. completions, embeddings) can follow in later phases.
- Phase 2 scope: intercept, redact, and proxy **OpenAI `/v1/chat/completions`** (streaming and non-streaming).

---

## 4. Streaming strategy

**Question:** Support Server-Sent Events (SSE) streaming from Day 1 of Phase 2?

**Decision:** **Yes. SSE streaming from Day 1.**

- **Rationale:** Non-streaming is simpler; streaming is where UX and product value are. Implementing streaming from the start avoids a later, harder retrofit.
- Phase 2 scope: support **SSE streaming** for chat completions (parse chunks, apply re-identification/masking where needed, forward SSE to the client).
- Non-streaming remains supported as a subset.

---

## Summary

| Prerequisite           | Choice / result |
|------------------------|------------------|
| Benchmark &lt;5ms       | **No** — mean 14.28ms, P95 25.47ms; mitigate later (quantize/smaller model). |
| Infrastructure (Phase 3) | **Local Docker** (Redis/Postgres); config **cloud-agnostic via env vars**. |
| Priority endpoint      | **OpenAI `/v1/chat/completions`** first. |
| Streaming              | **SSE from Day 1** (streaming + non-streaming). |

Phase 2 can proceed with: **OpenAI /v1/chat/completions**, **SSE streaming from Day 1**, and awareness that NER latency may require optimization in Phase 2 or Phase 6.
