/**
 * @unblockllm/sdk — OpenAI-compatible client that routes through the unblockllm proxy.
 * No PII in logs; default base URL uses https. Respects Phase 3 env (REDIS_URL, DATABASE_URL are proxy-side).
 */

import OpenAI from "openai";

const DEFAULT_BASE_URL =
  process.env.UNBLOCKLLM_BASE_URL ?? "https://127.0.0.1:8080";
const DEFAULT_API_KEY =
  process.env.OPENAI_API_KEY ?? process.env.UNBLOCKLLM_API_KEY ?? "not-set";

function normalizeBaseUrl(url: string): string {
  const u = url.trim().replace(/\/+$/, "");
  return u.startsWith("http://") || u.startsWith("https://") ? u : `https://${u}`;
}

export interface UnblockClientOptions {
  /** Proxy base URL (default: UNBLOCKLLM_BASE_URL or https://127.0.0.1:8080) */
  baseURL?: string;
  /** API key (default: OPENAI_API_KEY or UNBLOCKLLM_API_KEY) */
  apiKey?: string;
  /** Additional OpenAI client options */
  [key: string]: unknown;
}

/**
 * OpenAI-compatible client that sends requests to the unblockllm proxy.
 * The proxy redacts PII locally before forwarding to the LLM provider.
 */
export class UnblockClient {
  readonly client: OpenAI;
  readonly baseURL: string;

  constructor(options: UnblockClientOptions = {}) {
    const baseURL = normalizeBaseUrl(options.baseURL ?? DEFAULT_BASE_URL);
    const apiKey = options.apiKey ?? DEFAULT_API_KEY;
    this.baseURL = baseURL;
    this.client = new OpenAI({
      baseURL: `${baseURL}/v1`,
      apiKey,
      ...options,
    });
  }

  /** Chat completions API (OpenAI-compatible). */
  get chat(): OpenAI["chat"] {
    return this.client.chat;
  }

  /** Placeholder for Phase 5: generate proxy API key. */
  static generateApiKeyPlaceholder(): string {
    return "Use OPENAI_API_KEY or UNBLOCKLLM_API_KEY (Phase 5: key generation).";
  }
}

export default UnblockClient;
