"""Thin HTTP client to the Noesis REST API — Hermes plugin transport.

This module is the single transport layer. It never imports Noesis code directly;
all communication goes through the Noesis HTTP API so the plugin can run in any
Hermes environment (even a different machine) with just stdlib + the plugin.py.

API Base URL resolution (first hit wins):
  1. NOESIS_API_URL env var
  2. http://127.0.0.1:8647 (default)
"""
from __future__ import annotations

import json
import logging
import os
import urllib.error
import urllib.parse
import urllib.request
from typing import Any

logger = logging.getLogger("noesis-hermes.client")

DEFAULT_API_URL = "http://127.0.0.1:8647"
REQUEST_TIMEOUT = 15.0


def _api_url() -> str:
    return os.environ.get("NOESIS_API_URL", DEFAULT_API_URL).rstrip("/")


def _api(
    method: str,
    path: str,
    payload: dict | None = None,
    params: dict | None = None,
    timeout: float = REQUEST_TIMEOUT,
) -> dict:
    """One round-trip to the Noesis API. HTTP errors raise RuntimeError."""
    url = _api_url() + path
    if params:
        url += "?" + urllib.parse.urlencode(params)
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(
        url,
        data=data,
        method=method,
        headers={"Content-Type": "application/json"} if data is not None else {},
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return json.loads(resp.read().decode() or "{}")
    except urllib.error.HTTPError as e:
        try:
            detail = json.loads(e.read().decode()).get("detail", "")
        except Exception:
            detail = ""
        raise RuntimeError(f"HTTP {e.code} {path}: {detail or e.reason}") from None


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def health() -> dict:
    """GET /api/health — check if the Noesis daemon is running."""
    return _api("GET", "/api/health")


def ingest(text: str, source: str = "hermes") -> dict:
    """POST /api/ingest — inject raw text into the cognition pipeline."""
    return _api("POST", "/api/ingest", {"text": text, "source": source})


def stats() -> dict:
    """GET /api/stats — system statistics."""
    return _api("GET", "/api/stats")


def signal_stats() -> dict:
    """GET /api/stats/signals — per-signal-type counts."""
    return _api("GET", "/api/stats/signals")


def inject_signal(signal_type: str, payload: dict | None = None) -> dict:
    """POST /api/signals/inject — inject an arbitrary signal."""
    return _api("POST", "/api/signals/inject", {
        "signal_type": signal_type,
        "payload": payload or {},
    })


def observability() -> dict:
    """GET /api/observability/overview — system observability."""
    return _api("GET", "/api/observability/overview")


def noesis_available() -> bool:
    """Check if the Noesis daemon is reachable."""
    try:
        h = health()
        return h.get("status") == "ok"
    except Exception:
        return False
