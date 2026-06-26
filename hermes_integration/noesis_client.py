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


def memories() -> dict:
    """GET /api/memories — memory field state."""
    return _api("GET", "/api/memories")


def graph() -> dict:
    """GET /api/graph — knowledge graph state."""
    return _api("GET", "/api/graph")


def graph_expand(entity: str) -> dict:
    """GET /api/graph/expand — expand an entity with its relations."""
    return _api("GET", "/api/graph/expand", params={"entity": entity})


def recall(query: str, k: int = 6, mode: str = "fast") -> list[dict]:
    """Semantic retrieval over Noesis memory + knowledge graph.

    Searches both the graph entities and stored episodes
    for content matching the query. Returns scored results.
    """
    results: list[dict] = []
    query_lower = query.lower()

    # 1. Search graph entities
    try:
        g = graph()
        entities = g.get("graph", {}).get("entities", []) if "graph" in g else g.get("entities", [])
        for e in entities:
            name = (e.get("name", "") or "").lower()
            desc = (e.get("description", "") or "").lower()
            if query_lower in name or query_lower in desc:
                results.append({
                    "id": e.get("id", ""),
                    "text": e.get("name", ""),
                    "score": 1.0 if query_lower in name else 0.6,
                    "source": "noesis:graph",
                    "metadata": {"category": e.get("category", "")},
                })
    except Exception as exc:
        logger.debug("graph recall failed: %s", exc)

    # 2. Search episodes in memory
    try:
        m = memories()
        episodes = m.get("state", {}).get("episodes", [])
        for ep in episodes:
            content = ep.get("content", "") or ""
            if query_lower in content.lower():
                results.append({
                    "id": ep.get("id", ""),
                    "text": content[:400],
                    "score": 0.8,
                    "source": "noesis:memory",
                    "metadata": {"timestamp": str(ep.get("timestamp", ""))},
                })
    except Exception as exc:
        logger.debug("memory recall failed: %s", exc)

    # Sort by score descending, limit to k
    results.sort(key=lambda x: x.get("score", 0), reverse=True)
    return results[:k]


def noesis_available() -> bool:
    """Check if the Noesis daemon is reachable."""
    try:
        h = health()
        return h.get("status") == "ok"
    except Exception:
        return False


# ---------------------------------------------------------------------------
# Deep Observability Detail Endpoints
# ---------------------------------------------------------------------------


def identity_detail() -> dict:
    """GET /api/identity/detail — deep identity observability (beliefs, values, traits, roles, etc.)."""
    return _api("GET", "/api/identity/detail")


def memory_detail() -> dict:
    """GET /api/memory/detail — deep memory observability (working, episodic, semantic, procedural, etc.)."""
    return _api("GET", "/api/memory/detail")


def executive_detail() -> dict:
    """GET /api/executive/detail — deep executive observability (goals, projects, tasks, plans, etc.)."""
    return _api("GET", "/api/executive/detail")


def awareness_detail() -> dict:
    """GET /api/awareness/detail — deep awareness observability (observer, attention, health, curiosity, etc.)."""
    return _api("GET", "/api/awareness/detail")


def simulation_detail() -> dict:
    """GET /api/simulation/detail — deep simulation observability (scenarios, assumptions, forecasts, risks, etc.)."""
    return _api("GET", "/api/simulation/detail")


def core_detail() -> dict:
    """GET /api/core/detail — deep core system observability (event_bus, scheduler, registry, metrics, etc.)."""
    return _api("GET", "/api/core/detail")
