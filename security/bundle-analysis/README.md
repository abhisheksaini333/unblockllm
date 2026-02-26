# Dashboard bundle analysis (PERF-03)

Run from repo root:

```bash
cd apps/dashboard && npm run analyze
```

This runs `ANALYZE=true next build`, which opens the bundle analyzer report in the browser after the build. Use it to:

- Confirm no sensitive logic (e.g. masking) is shipped to client-side JS.
- Identify large dependencies and opportunities to reduce bundle size.

Reports are not committed; generate locally when needed.
