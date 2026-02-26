# Security (Phase 6)

- **pen-test/** — Penetration test scripts (synthetic data only) and REPORT.md.
- **load-test/** — k6 load test script and REPORT.md.
- **sbom.json** — CycloneDX SBOM for Dashboard (npm). Generate with:
  ```bash
  cd apps/dashboard && npm sbom --sbom-format=cyclonedx > ../../security/sbom.json
  ```
- **Dependency scans:**
  - Rust: `cargo audit` (from repo root). One known **medium**: rsa via sqlx-mysql (transitive; we use postgres only; no upgrade path).
  - Node: `npm audit` in `apps/dashboard` (address Critical/High; some require major upgrades).
  - Python: `pip audit` or `safety check` in `python/` if available.

**Target: Zero critical vulnerabilities.** Current: 0 critical in Rust; npm may report high/critical in dev deps (e.g. eslint/minimatch)—fix or accept for dev-only.
