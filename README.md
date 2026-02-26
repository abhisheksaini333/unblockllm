# unblockllm

**Zero-Trust Privacy Proxy for LLMs**

Enable enterprises to use public LLMs (OpenAI, Anthropic) without leaking PII.  
Local PII redaction (ONNX) → anonymized prompt → response → re-identify → return.

- **PII never leaves local infrastructure unencrypted.**
- **Target overhead:** &lt;20ms (mask + unmask).
- **Compliance:** GDPR, HIPAA, SOC2-ready (metadata-only audit).

## Repository structure

```
unblockllm/
├── crates/           # Rust core (proxy, ONNX integration)
├── python/           # Python SDK + benchmark + CLI
├── apps/             # Next.js dashboard
├── proto/            # Protobuf schemas (internal comms)
├── .github/workflows # CI/CD
└── docs/             # Architecture, security model
```

## Phase 1 deliverable

- Monorepo with Cargo workspace, Python package, Next.js app scaffold.
- Protobuf schemas for redaction/re-identification payloads.
- ONNX NER model selected and benchmarked (&lt;5ms inference target).
- GitHub Actions: build, test, lint, security scan.

## License

See [LICENSE](LICENSE).
