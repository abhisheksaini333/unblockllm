"""Integration tests for unblockllm-scan CLI. Synthetic data only; no PII in repo."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

from typer.testing import CliRunner

from unblockllm.cli.scan import app
from unblockllm.pii import find_pii_spans

runner = CliRunner()


def test_scan_file_json_stdout() -> None:
    """CLI scans file and outputs JSON to stdout."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        f.write("Contact support@example.com or 555-123-4567 for help.\n")
        path = Path(f.name)
    try:
        result = runner.invoke(app, ["--file", str(path)])
        assert result.exit_code == 0
        data = json.loads(result.stdout)
        assert "summary" in data
        assert data["summary"]["total_findings"] == 2
        assert data["summary"]["by_type"]["EMAIL"] == 1
        assert data["summary"]["by_type"]["PHONE"] == 1
        assert len(data["findings"]) == 2
    finally:
        path.unlink(missing_ok=True)


def test_scan_file_html_output() -> None:
    """CLI writes HTML report to --output file."""
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        f.write("Email: user@test.com. SSN: 123-45-6789.\n")
        path = Path(f.name)
    out_path = path.with_suffix(".html")
    try:
        result = runner.invoke(app, ["--file", str(path), "--output", str(out_path), "--format", "html"])
        assert result.exit_code == 0
        assert out_path.exists()
        html = out_path.read_text()
        assert "Scan Result: 2 PII leak(s) found" in html
        assert "EMAIL" in html and "SSN" in html
    finally:
        path.unlink(missing_ok=True)
        out_path.unlink(missing_ok=True)


def test_scan_file_missing() -> None:
    """CLI exits non-zero when file does not exist."""
    result = runner.invoke(app, ["--file", "/nonexistent/path.txt"])
    assert result.exit_code == 1


def test_pii_find_spans_synthetic() -> None:
    """PII regex finds EMAIL, PHONE, SSN in synthetic text."""
    text = "Reach alice@example.com or 800-555-1234. SSN 111-22-3333."
    spans = find_pii_spans(text)
    types = {s.entity_type for s in spans}
    assert "EMAIL" in types
    assert "PHONE" in types
    assert "SSN" in types
    assert len(spans) >= 3
