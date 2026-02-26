# Incident Response Plan

Enterprise-ready procedure for security and availability incidents involving the unblockllm proxy or dashboard.

## 1. Kill Switch (Immediate)

- **Proxy:** Disable traffic to the proxy (e.g. remove from load balancer, scale to 0, or stop the service). This stops all PII-redacted traffic to upstream LLMs.
- **Dashboard:** If the incident involves the dashboard (e.g. auth bypass), take the dashboard out of rotation or disable login until patched.
- **Secrets:** Rotate any exposed API keys (e.g. `PROXY_API_KEY`) via Dashboard `POST /api/v1/keys/rotate` and update secret store (K8s Secrets / AWS Secrets Manager); restart proxy with new key.

## 2. Notify Customers

- Notify affected customers per contract and regulatory requirements (e.g. GDPR breach notification within 72 hours if personal data is affected).
- Use a prepared template (see below) and include: nature of the incident, scope, steps taken, and contact for questions.

**Breach notification template (summary):**

- Subject: Security Incident Notice – [Product/Service Name]
- We are writing to inform you of a security incident that may affect your data. We identified [brief description] on [date]. Our investigation shows [scope]. We have [actions taken: e.g. disabled access, rotated keys, applied patch]. We will [next steps: e.g. post-mortem, additional controls]. If you have questions, contact [email/portal].

## 3. Patch and Harden

- Apply fixes (code/config) in a staging environment; run tests and verification (see VERIFICATION_CHECKLIST.md).
- Deploy to production; re-enable proxy/dashboard only after validation.
- Update dependencies (`cargo audit`, `npm audit`) and document any accepted risks in `docs/SECURITY.md`.

## 4. Post-Mortem

- Document timeline, root cause, and corrective actions in an internal post-mortem.
- Update runbooks and this incident response plan if gaps are identified.
- Retain evidence (logs, audit records) for compliance; do not retain raw PII longer than retention policy (e.g. 90 days for audit logs).

---

**Contacts:** Define on-call and escalation contacts in your operations runbook. Ensure SOC2 evidence (CC2, CC4) is updated when incidents occur.
