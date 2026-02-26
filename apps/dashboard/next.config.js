const withSentryConfig = require("@sentry/nextjs").withSentryConfig;

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  reactStrictMode: true,
  async headers() {
    return [
      {
        source: "/:path*",
        headers: [
          { key: "X-Frame-Options", value: "DENY" },
          { key: "X-Content-Type-Options", value: "nosniff" },
          ...(process.env.NODE_ENV === "production"
            ? [{ key: "Strict-Transport-Security", value: "max-age=31536000; includeSubDomains" }]
            : []),
        ],
      },
    ];
  },
};

// PERF-03: Bundle analyzer — ANALYZE=true npm run build opens report in browser
const withBundleAnalyzer = require("@next/bundle-analyzer")({
  enabled: process.env.ANALYZE === "true",
});

// Sentry: optional; set NEXT_PUBLIC_SENTRY_DSN / SENTRY_DSN to enable
const sentryOptions = { silent: true, widenClientFileUpload: true };
module.exports = withSentryConfig(
  withBundleAnalyzer(nextConfig),
  sentryOptions
);
