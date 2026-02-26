# Agent Skills (Activated for Production Readiness Verification)

These skills are **installed locally** under `.agents/skills/` and should be loaded when running production readiness checks, security audits, or compliance validation.

---

## Instruction → Skill Mapping

Use **find-skills** to discover more; apply the skills below per the instruction:

| Instruction Category | Sub-topics | Local Skill(s) | Optional External Install |
|----------------------|------------|----------------|----------------------------|
| **Security Engineering** | OWASP, PII Protection, Secret Management, Pen Testing | owasp-security, security-compliance, **pen-testing** | `npx skills add aj-geddes/useful-ai-prompts@penetration-testing` |
| **Rust Best Practices** | clippy, unwrap/expect, error handling, async | rust-best-practices | — |
| **Compliance** | SOC2, GDPR, SBOM, Audit Logging | security-compliance, security-compliance-audit | — |
| **DevOps** | Docker, Multi-arch, CI/CD, Non-root containers | docker-expert | `npx skills add davila7/claude-code-templates@senior-devops` |
| **Performance** | Latency benchmarking, Load testing | performance-optimization | `npx skills add dengineproblem/agents-monorepo@k6-load-test` |
| **Verification & Testing** | Automated checks, Manual validation, Pass/Fail criteria | **verification-testing** | `npx skills add affaan-m/everything-claude-code@verification-loop` or `yonatangross/orchestkit@verify` |

---

## Local Skills (by path)

| Skill | Path | Use When |
|-------|------|----------|
| **find-skills** | `find-skills/SKILL.md` | Discovering/installing external skills via `npx skills find` / `npx skills add` |
| **rust-best-practices** | `rust-best-practices/SKILL.md` | Writing/reviewing Rust: clippy, no unwrap/expect, Result handling, error propagation, async |
| **owasp-security** | `owasp-security/SKILL.md` | OWASP Top 10, injection, auth, rate limiting, secure coding, PII in logs |
| **security-compliance** | `security-compliance/SKILL.md` | SOC2, GDPR, defense-in-depth, threat modeling, compliance lifecycle, secret management |
| **security-compliance-audit** | `security-compliance-audit/SKILL.md` | SOC2/GDPR/HIPAA/PCI-DSS audits, control assessment, evidence, SBOM, audit logging |
| **docker-expert** | `docker-expert/SKILL.md` | Dockerfiles: non-root, multi-stage, multi-arch, security hardening |
| **performance-optimization** | `performance-optimization/SKILL.md` | Latency budgets, perceived performance, benchmarking |
| **verification-testing** | `verification-testing/SKILL.md` | Automated checks, manual validation, pass/fail criteria; see `docs/VERIFICATION_CHECKLIST.md` |
| **pen-testing** | `pen-testing/SKILL.md` | Pen testing, PII protection checks, log injection, API abuse; see `security/pen-test/` |

---

## Activation (summary)

When executing verification (e.g. PRODUCTION_READINESS_REPORT or VERIFICATION_CHECKLIST):

1. **Rust** — `rust-best-practices`: `cargo clippy --all-targets -- -D warnings`, no `unwrap()`/`expect()` in production.
2. **Security** — `owasp-security`, `security-compliance`, `pen-testing`: PII protection, rate limiting, audit logs, no secrets in repo, run `security/pen-test/*.sh` as needed.
3. **Compliance** — `security-compliance-audit`: SBOM, COMPLIANCE_PACKET accuracy, control evidence, audit logging.
4. **DevOps** — `docker-expert`: non-root USER, multi-arch docs, distroless/minimal base; optional senior-devops for CI/CD.
5. **Performance** — `performance-optimization` + project SKILL.md: &lt;20ms overhead, latency benchmark scope; optional k6-load-test for load tests.
6. **Verification** — `verification-testing`: use VERIFICATION_CHECKLIST.md format (command, expected output, fail condition, fix, file to inspect, status).

All local skills are under `.agents/skills/<name>/SKILL.md`. Reference these files when generating or validating verification steps.
