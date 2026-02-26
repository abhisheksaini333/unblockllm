"""
UnblockClient: OpenAI-compatible client that routes requests through the unblockllm proxy.

Uses base_url to point to the proxy (default https). No PII in logs; API key from env.
"""

from __future__ import annotations

import os
from typing import Any, Optional

# Optional: only import when openai is installed
try:
    from openai import OpenAI
except ImportError:
    OpenAI = None  # type: ignore[misc, assignment]


def _default_base_url() -> str:
    """Default proxy URL. Prefer https."""
    return os.environ.get("UNBLOCKLLM_BASE_URL", "https://127.0.0.1:8080")


def _default_api_key() -> Optional[str]:
    """API key from env. Phase 5 may add proxy-specific key generation."""
    return os.environ.get("OPENAI_API_KEY") or os.environ.get("UNBLOCKLLM_API_KEY")


class UnblockClient:
    """
    OpenAI-compatible client that sends requests to the unblockllm proxy.

    The proxy redacts PII locally before forwarding to the LLM provider.
    Default base_url uses https when possible (see UNBLOCKLLM_BASE_URL).
    """

    def __init__(
        self,
        base_url: Optional[str] = None,
        api_key: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        if OpenAI is None:
            raise ImportError("openai package is required. Install with: pip install openai")
        self._base_url = (base_url or _default_base_url()).rstrip("/")
        if not self._base_url.startswith("https://") and not self._base_url.startswith("http://"):
            self._base_url = "https://" + self._base_url
        self._api_key = api_key or _default_api_key()
        self._client = OpenAI(
            base_url=f"{self._base_url}/v1",
            api_key=self._api_key or "not-set",
            **kwargs,
        )

    @property
    def chat(self) -> Any:
        """OpenAI-style chat completions API."""
        return self._client.chat

    @property
    def client(self) -> Any:
        """Underlying OpenAI client for advanced use."""
        return self._client

    @classmethod
    def generate_api_key_placeholder(cls) -> str:
        """
        Placeholder for Phase 5: generate proxy API key.
        For now, use OPENAI_API_KEY or UNBLOCKLLM_API_KEY.
        """
        return "Use OPENAI_API_KEY or UNBLOCKLLM_API_KEY (Phase 5: key generation)."
