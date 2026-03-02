// k6 load test for UnblockLLM proxy.
// Requires the proxy running at PROXY_URL (default http://127.0.0.1:8080)
// pointing at a mock OpenAI backend (or real, if key has quota).
//
// Usage:
//   # Start proxy + mock first, then:
//   k6 run tests/load/k6_proxy.js
//   k6 run --vus 10 --duration 30s tests/load/k6_proxy.js
//   k6 run tests/load/k6_proxy.js --env PROXY_URL=http://host:port

import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Trend } from "k6/metrics";

const proxyUrl = __ENV.PROXY_URL || "http://127.0.0.1:8080";

// Custom metrics
const errorRate = new Rate("error_rate");
const latency = new Trend("proxy_latency_ms");

// Scenarios: ramp up from 1 to 20 VUs over 1 minute
export const options = {
  scenarios: {
    ramp: {
      executor: "ramping-vus",
      startVUs: 1,
      stages: [
        { duration: "10s", target: 5 },
        { duration: "20s", target: 10 },
        { duration: "20s", target: 20 },
        { duration: "10s", target: 0 },
      ],
    },
  },
  thresholds: {
    http_req_duration: ["p(95)<500", "p(99)<1000"],
    error_rate: ["rate<0.05"],
  },
};

// PII payloads to cycle through
const payloads = [
  {
    model: "gpt-4o-mini",
    messages: [
      {
        role: "user",
        content:
          "My email is user@example.com and phone 555-123-4567. Say hello.",
      },
    ],
    stream: false,
  },
  {
    model: "gpt-4o-mini",
    messages: [
      {
        role: "user",
        content:
          "John Smith from Microsoft in Seattle needs help. SSN 123-45-6789.",
      },
    ],
    stream: false,
  },
  {
    model: "gpt-4o-mini",
    messages: [
      { role: "user", content: "What is the capital of France?" },
    ],
    stream: false,
  },
  {
    model: "gpt-4o-mini",
    messages: [
      {
        role: "user",
        content:
          "Dr. Jane Doe at University of California, Los Angeles. Email jane@university.edu.",
      },
    ],
    stream: true,
  },
];

export default function () {
  // Health check (fast)
  const healthRes = http.get(`${proxyUrl}/health`);
  check(healthRes, {
    "health 200": (r) => r.status === 200,
  });

  // Chat completion with PII
  const payload = payloads[Math.floor(Math.random() * payloads.length)];
  const params = {
    headers: {
      "Content-Type": "application/json",
      Authorization: "Bearer sk-test",
    },
    timeout: "10s",
  };

  const start = Date.now();
  const res = http.post(
    `${proxyUrl}/v1/chat/completions`,
    JSON.stringify(payload),
    params
  );
  const elapsed = Date.now() - start;

  latency.add(elapsed);
  errorRate.add(res.status >= 500);

  check(res, {
    "status is 200": (r) => r.status === 200,
    "has body": (r) => r.body && r.body.length > 0,
    "latency < 500ms": () => elapsed < 500,
  });

  sleep(0.1); // small pause between requests
}
