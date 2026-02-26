---
name: verification-testing
description: Apply automated checks, manual validation steps, and clear pass/fail criteria when verifying production readiness, security, or compliance. Use when running verification checklists, defining acceptance criteria, or validating fixes.
---

# Verification & Testing

Use this skill when executing or designing **production readiness verification**, **manual validation**, and **pass/fail criteria** for security, compliance, and code quality.

## When to Use

- Running the production readiness verification checklist (Section 1–8).
- Defining expected output, fail conditions, and fix commands for each check.
- Validating that code/docs match claims (e.g. no unwrap in production, SBOM referenced).
- Manual validation steps (e.g. start proxy, send request, confirm logs show masked PII only).
- CI or local pass/fail gates (clippy, audit, tests, lint, build).

## Principles

1. **One check, one command** — Each verification item has an exact command to run.
2. **Expected vs fail** — Document what “pass” looks like and what indicates failure.
3. **Fix path** — If it fails, document the fix (command or code change).
4. **File to inspect** — Point to the file(s) and what to look for.
5. **Status** — Record ⬜ Not Run / ✅ Pass / ❌ Fail after execution.

## Output Format (per item)

```markdown
### [Item]: [Name]

**Command:**
\`\`\`bash
[exact command]
\`\`\`

**Expected Output:**
[what indicates pass]

**Fail Condition:**
[what indicates a problem]

**Fix (if needed):**
[exact fix command or code change]

**File to Inspect:**
- `path/to/file` — Look for: [specific content]

**Status:** ⬜ Not Run / ✅ Pass / ❌ Fail
```

## Execution Order

Run verification in order: Section 1 (Rust) → 2 (SBOM) → 3 (Env/Secrets) → 4 (Dashboard) → 5 (Docker) → 6 (Security/Privacy) → 7 (Performance) → 8 (Compliance docs). Stop after each section for confirmation if the user requested step-by-step validation.

## Project Reference

- **Checklist:** `docs/VERIFICATION_CHECKLIST.md` — Full commands, expected output, and status.
- **Report:** `docs/PRODUCTION_READINESS_REPORT.md` — Verdict and checklist results.

## Optional External Skills

For more formal QA/verification workflows:
- `npx skills add affaan-m/everything-claude-code@verification-loop`
- `npx skills add yonatangross/orchestkit@verify`
