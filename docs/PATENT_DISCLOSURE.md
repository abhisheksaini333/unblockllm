# Patent Disclosure: Stateful Re-identification in Streaming LLM Proxies

**Confidential — For legal counsel review.**

## Title

**Stateful Re-identification in Streaming LLM Proxies with Local PII Redaction.**

## Field

The disclosure relates to privacy-preserving proxy systems that sit between client applications and large language model (LLM) APIs, and in particular to methods and systems for redacting personally identifiable information (PII) from outbound requests and re-inserting it into streaming responses without persisting PII.

## Background

Many applications send user-generated text to third-party LLM APIs. Such text often contains PII (e.g., names, emails, phone numbers). Sending raw PII to LLM providers creates privacy and compliance risk. Simply stripping PII from requests degrades utility when the user expects a response that refers back to their data (e.g., “Send an email to John at john@example.com”). A need exists for a proxy that (1) redacts PII from the request before forwarding, (2) forwards the redacted request and receives a streaming response, and (3) re-identifies the response by replacing placeholders with the original PII in a way that does not require storing PII durably and that works with streaming delivery.

## Summary of the Invention

A proxy system is interposed between clients and an LLM API. The proxy (1) receives a request containing message content; (2) detects PII in the content using local detection (e.g., regex and/or NER); (3) replaces each PII span with a placeholder (e.g., `[EMAIL_1]`) and builds an ephemeral mapping from placeholder to original value; (4) forwards the redacted request to the LLM and receives a streaming response; (5) as response chunks arrive, replaces any placeholders with the corresponding original values using the mapping; and (6) streams the re-identified response to the client. The mapping is stored only in ephemeral storage (e.g., in-memory or short-TTL cache keyed by request ID) and is not written to durable audit logs. Audit logs record only metadata (e.g., request ID, entity type names, counts)—never raw PII or mapping values.

## Claims (Draft)

1. **A method for privacy-preserving proxying of requests to a language model API**, comprising: receiving a request comprising message content; detecting one or more PII spans in the message content; replacing each PII span with a placeholder to form redacted content; storing a mapping from each placeholder to the corresponding original PII in ephemeral storage keyed by a request identifier; sending the redacted content to the language model API; receiving a streaming response from the language model API; for each chunk of the streaming response, replacing any placeholder present in the chunk with the corresponding original PII from the mapping; and streaming the re-identified chunks to the client; wherein the mapping is not persisted to durable storage and is discarded after the response is complete.

2. **The method of claim 1**, wherein detecting PII comprises at least one of: applying one or more regular expressions, or running a named-entity recognition model locally.

3. **The method of claim 1**, wherein the ephemeral storage comprises an in-memory store or a cache with a time-to-live (TTL) of less than five minutes.

4. **The method of claim 1**, further comprising writing audit metadata to durable storage, the audit metadata comprising at least one of: request identifier, count of redacted entities, or list of entity type names; and wherein the audit metadata does not include any raw PII or any placeholder-to-value mapping.

5. **A system** comprising one or more processors and memory storing instructions that, when executed, cause the system to perform the method of any of claims 1–4.

## Prior Art Considerations

- Generic HTTP proxies and API gateways do not perform PII redaction and streaming re-identification with ephemeral mapping.
- Static PII redaction tools (e.g., log scrubbers) do not address the streaming response re-identification problem.
- Client-side redaction alone does not protect against accidental inclusion of PII in prompts; a server-side proxy ensures all traffic to the LLM is redacted.

## Use in Product

unblockllm implements this approach: the Rust proxy performs local NER/regex detection, masking, and re-identification using Redis (or in-memory) mapping with 60-second TTL; audit logs in PostgreSQL store only request_id, entity_count, and entity_types.

---

*This document is a draft disclosure for patent counsel. It is not a filed application and does not constitute legal advice.*
