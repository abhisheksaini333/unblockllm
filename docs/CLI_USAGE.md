# unblockllm-scan CLI Usage

The **Viral CLI Scanner** scans files for PII (EMAIL, PHONE, SSN) using regex. **All processing is local; no data leaves your machine.**

## Install

From the repo root:

```bash
cd python && pip install -e ".[dev]"
```

Then run:

```bash
unblockllm-scan --help
```

## Commands

### Scan a file

```bash
unblockllm-scan --file logs.json
```

Output is JSON to stdout by default, with a summary and list of findings (type, start, end). No raw PII values are printed in the default JSON report.

### Write report to a file

```bash
unblockllm-scan --file logs.json --output report.json
unblockllm-scan --file logs.json --output report.html --format html
```

- **`--output` / `-o`:** Write the report to this path.
- **`--format` / `-F`:** `json` (default) or `html`.

### Shareable output

The CLI prints a one-line summary suitable for sharing, e.g.:

```
Scan Result: 3 PII leak(s) found.
```

When writing to a file, the HTML report includes the same summary in the heading: **Scan Result: N PII leak(s) found**.

## Detected types

| Type  | Description |
|-------|-------------|
| EMAIL | Email addresses (regex) |
| PHONE | US-style phone numbers (10 digits, optional separators) |
| SSN  | US SSN pattern XXX-XX-XXXX |

Detection is regex-based only (aligned with Phase 2 proxy masking). No NER model or network calls are used.

## Offline safety

- No data is sent to any server.
- All processing runs on the file content in memory on your machine.
- Safe to run on sensitive logs; results are only in the report you choose to write.

## Example

```bash
# Create a small test file
echo 'Contact support@example.com or 555-123-4567.' > /tmp/sample.txt

# Scan and print JSON
unblockllm-scan --file /tmp/sample.txt

# Scan and write HTML report
unblockllm-scan --file /tmp/sample.txt --output report.html --format html
```
