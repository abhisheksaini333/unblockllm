# Launch Blog Post (Draft)

**Title:** 40% of AI Apps Leak PII—Here’s How We Fix It

**Subtitle:** Introducing unblockllm: a Zero-Trust proxy that redacts PII before it ever reaches an LLM.

---

Studies and audits suggest that a large share of applications that send user data to language model APIs inadvertently expose personally identifiable information (PII). Whether it’s support chatbots, writing assistants, or internal tools, the moment you paste a name, email, or phone number into a prompt, that data can end up in provider logs, training pipelines, or third-party systems. For many teams, that’s a dealbreaker for adopting LLMs in production.

**unblockllm** is an open-source, Zero-Trust privacy proxy that sits between your app and any OpenAI-compatible API. It doesn’t just log or warn—it **redacts PII locally** before a single byte reaches the LLM. Names, emails, phone numbers, and SSNs are replaced with placeholders like `[EMAIL_1]` so the model never sees the real data. When the response streams back, the proxy **re-identifies** the text in real time, swapping placeholders back so your user sees a natural reply. The mapping is ephemeral (we use a short-lived cache); we never write raw PII or mapping values to disk. Our audit logs store only metadata: request IDs, entity counts, and type names—enough for compliance, zero PII.

We built it for three constraints: **privacy** (no PII in logs or durable storage), **speed** (under 20ms overhead so it’s viable in production), and **compatibility** (drop-in for any OpenAI-compatible client). You get a single binary or Docker image; point your app at the proxy instead of the LLM; and you can start scanning your own logs with our CLI, **unblockllm-scan**, to see how much PII is sitting in plain text. The scan runs entirely on your machine—nothing is uploaded.

If you’re shipping AI features and care about compliance (SOC2, GDPR, or just “don’t send customer data to third parties”), try unblockllm. Open source on GitHub; Enterprise options include a dashboard, remote policy, and audit visualization.

**Scan your logs. Unblock AI adoption.**

- [GitHub](https://github.com/unblockllm/unblockllm)
- [Docs](https://github.com/unblockllm/unblockllm#readme)

---

*Draft for marketing/GTM. Adjust “40%” and links per your sources and deployment.*
