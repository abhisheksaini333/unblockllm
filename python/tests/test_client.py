"""Unit tests for UnblockClient. Mock network calls; no PII in assertions."""

from __future__ import annotations

import os
from unittest.mock import patch



def test_unblock_client_base_url_default() -> None:
    """UnblockClient uses UNBLOCKLLM_BASE_URL or https default."""
    from unblockllm import UnblockClient

    with patch.dict(os.environ, {"UNBLOCKLLM_BASE_URL": "https://proxy.example.com"}, clear=False):
        client = UnblockClient()
        assert "proxy.example.com" in client._base_url
        assert client._base_url.startswith("https://")


def test_unblock_client_base_url_explicit() -> None:
    """UnblockClient accepts explicit base_url and normalizes to https."""
    from unblockllm import UnblockClient

    client = UnblockClient(base_url="http://127.0.0.1:8080")
    assert "127.0.0.1" in client._base_url
    client2 = UnblockClient(base_url="127.0.0.1:9999")
    assert client2._base_url.startswith("https://")
    assert "9999" in client2._base_url


def test_unblock_client_has_chat() -> None:
    """UnblockClient exposes chat completions API."""
    from unblockllm import UnblockClient

    client = UnblockClient(base_url="https://127.0.0.1:8080")
    assert client.chat is not None
    assert hasattr(client.chat, "completions")


def test_generate_api_key_placeholder() -> None:
    """Placeholder helper returns instructions string."""
    from unblockllm import UnblockClient

    out = UnblockClient.generate_api_key_placeholder()
    assert isinstance(out, str)
    assert "OPENAI_API_KEY" in out or "UNBLOCKLLM" in out
