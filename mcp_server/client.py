"""Thin HTTP client to the Noesis REST API — MCP transport.

Every operation goes through the Noesis API on :8647.
This is a separate thin client (not importing the Hermes plugin version)
so the MCP server has no dependency on Hermes internals.
"""
from __future__ import annotations

import json
import os
import urllib.error
import urllib.request

API_URL = os.environ.get("NOESIS_API_URL", "http://127.0.0.1:8647").rstrip("/")
TIMEOUT = 15.0


def _api(method: str, path: str, payload: dict | None = None) -> dict:
    url = API_URL + path
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(
        url, data=data, method=method,
        headers={"Content-Type": "application/json"} if data is not None else {},
    )
    try:
        with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
            return json.loads(resp.read().decode() or "{}")
    except urllib.error.HTTPError as e:
        detail = ""
        try:
            detail = json.loads(e.read().decode()).get("detail", "")
        except Exception:
            pass
        raise RuntimeError(f"HTTP {e.code}: {detail or e.reason}") from None


def health() -> dict:
    return _api("GET", "/api/health")


def ingest(text: str, source: str = "mcp") -> dict:
    return _api("POST", "/api/ingest", {"text": text, "source": source})


def stats() -> dict:
    return _api("GET", "/api/stats")


def signal_stats() -> dict:
    return _api("GET", "/api/stats/signals")


def observability() -> dict:
    return _api("GET", "/api/observability/overview")


def memories() -> dict:
    return _api("GET", "/api/memories")


def graph() -> dict:
    return _api("GET", "/api/graph")


def identity() -> dict:
    return _api("GET", "/api/identity")


def recall(query: str, k: int = 6) -> dict:
    """Noesis recall — searches through field cache for relevant context."""
    result: dict = {"results": [], "query": query, "total": 0}

    # Search graph entities
    try:
        g = graph()
        entities = _safe_get(g, ["graph", "entities"], [])
        query_lower = query.lower()
        matches = []
        for e in entities:
            name = _safe_get(e, ["name"], "").lower()
            desc = _safe_get(e, ["description"], "").lower()
            if query_lower in name or query_lower in desc:
                matches.append({
                    "id": _safe_get(e, ["id"], ""),
                    "text": _safe_get(e, ["name"], ""),
                    "type": _safe_get(e, ["category"], "entity"),
                    "score": 1.0 if query_lower in name else 0.6,
                    "source": "noesis:graph",
                })
        matches.sort(key=lambda x: x["score"], reverse=True)
        result["results"].extend(matches[:k])
    except Exception:
        pass

    # Search memories
    try:
        m = memories()
        episodes = _safe_get(m, ["state", "episodes"], [])
        for ep in episodes:
            content = _safe_get(ep, ["content"], "")
            if query_lower in content.lower():
                result["results"].append({
                    "id": _safe_get(ep, ["id"], ""),
                    "text": content[:300],
                    "type": "episode",
                    "score": 0.8,
                    "source": "noesis:memory",
                })
    except Exception:
        pass

    result["total"] = len(result["results"])
    return result


def _safe_get(d: dict, keys: list[str], default=None):
    current = d
    for k in keys:
        if isinstance(current, dict):
            current = current.get(k, {})
        else:
            return default
    return current if current != {} else default
