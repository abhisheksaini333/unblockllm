/**
 * k6 load test: Proxy /v1/chat/completions at target 1000 RPS.
 * Measures latency (p50, p95, p99) and error rate. Overhead target <20ms (p95).
 *
 * Run: k6 run security/load-test/load_test.js
 * Env: PROXY_URL (default http://127.0.0.1:8080), OPENAI_API_KEY (required for real upstream; use sk-test for mock)
 */

import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Trend } from "k6/metrics";

const proxyUrl = __ENV.PROXY_URL || "http://127.0.0.1:8080";
const apiKey = __ENV.OPENAI_API_KEY || "sk-test-key";

const errorRate = new Rate("errors");
const chatLatency = new Trend("chat_latency_ms");

export const options = {
  scenarios: {
    constant_rps: {
      executor: "constant-arrival-rate",
      rate: 1000,
      timeUnit: "1s",
      duration: "60s",
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
  },
  thresholds: {
    http_req_duration: ["p(95)<5000"],
    errors: ["rate<0.01"],
  },
};

export default function () {
  const url = `${proxyUrl}/v1/chat/completions`;
  const payload = JSON.stringify({
    model: "gpt-4o-mini",
    messages: [{ role: "user", content: "Say hello in one word." }],
    stream: false,
  });
  const params = {
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiKey}`,
    },
  };
  const res = http.post(url, payload, params);
  chatLatency.add(res.timings.duration);
  const ok = check(res, { "status is 200 or 429": (r) => r.status === 200 || r.status === 429 });
  if (!ok) errorRate.add(1);
  else errorRate.add(0);
  sleep(0.01);
}

// k6 prints default summary with p50/p95/p99. For proxy-only overhead use: cargo bench --bench latency
