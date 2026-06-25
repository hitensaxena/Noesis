"""Noesis MemoryProvider for Hermes Agent — HTTP-only transport.

Every operation goes through the Noesis API on :8647.
The plugin holds no database connection and imports no Noesis code.

Exposes register(ctx) which calls ctx.register_memory_provider().
"""
from __future__ import annotations

import json
import logging
import os
import re
import time
from typing import Any

from agent.memory_provider import MemoryProvider
from tools.registry import tool_error

import hermes_integration.noesis_client as noesis

logger = logging.getLogger("noesis-hermes.plugin")

# ---------------------------------------------------------------------------
# Tool schemas
# ---------------------------------------------------------------------------

RECALL_SCHEMA = {
    "name": "noesis_recall",
    "description": (
        "Retrieve memories and cognitive state from the Noesis system. "
        "Returns signal counts, processor stats, and system observability. "
        "Use for understanding current cognitive state and recent signal history."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "What to look for (passed to stats/observability).",
            },
        },
        "required": [],
    },
}

INGEST_SCHEMA = {
    "name": "noesis_ingest",
    "description": (
        "Record a new experience into the Noesis cognitive architecture. "
        "The text is injected into the signal cascade — it will be recorded as an episode, "
        "extracted into triples for the knowledge graph, checked for consolidation, "
        "and may trigger attention shifts, belief changes, and narrative generation."
        "Use for anything you want the system to remember and learn from."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "text": {
                "type": "string",
                "description": "The experience text to record.",
            },
        },
        "required": ["text"],
    },
}

STATUS_SCHEMA = {
    "name": "noesis_status",
    "description": (
        "Get the current status of the Noesis cognitive system. "
        "Returns health, uptime, field/processor/signal counts, and signal processing metrics. "
        "Use to check if the system is running and healthy."
    ),
    "parameters": {
        "type": "object",
        "properties": {},
        "required": [],
    },
}

# ---------------------------------------------------------------------------
# MemoryProvider implementation
# ---------------------------------------------------------------------------


class NoesisProvider(MemoryProvider):
    """MemoryProvider that delegates all operations to the Noesis API."""

    def __init__(self):
        self._initialized = False
        self._start_time = 0.0

    # ── lifecycle ────────────────────────────────────────────────────────

    def initialize(self, ctx: Any) -> None:
        logger.info("NoesisProvider initializing...")
        self._start_time = time.time()
        self._initialized = True
        logger.info("NoesisProvider initialized (API: %s)", noesis._api_url())

    def shutdown(self) -> None:
        logger.info("NoesisProvider shutting down")
        self._initialized = False

    # ── Hermes MemoryProvider interface ──────────────────────────────────

    def register_tools(self, ctx: Any) -> None:
        """Register Noesis tools with the Hermes agent."""
        logger.info("NoesisProvider registering tools...")
        ctx.register_tool(RECALL_SCHEMA, self.handle_recall)
        ctx.register_tool(INGEST_SCHEMA, self.handle_ingest)
        ctx.register_tool(STATUS_SCHEMA, self.handle_status)
        logger.info("NoesisProvider: 3 tools registered")

    def recall(self, query: str, k: int = 6, mode: str = "fast") -> list[dict]:
        """Semantic retrieval over the Noesis knowledge graph + memory.

        Searches through graph entities and stored episodes for content
        matching the query. Returns scored results with source metadata.
        """
        try:
            return noesis.recall(query, k=k, mode=mode)
        except Exception as e:
            logger.error("Noesis recall failed: %s", e)
            return []

    def context(self) -> list[str]:
        """Return context lines for prompt injection."""
        try:
            obs = noesis.observability()
            return [
                f"Noesis status: {obs.get('status', 'ok')}",
                f"Fields: {obs.get('fields', 0)}, "
                f"Processors: {obs.get('processors', 0)}, "
                f"Signal types: {obs.get('signal_types', 0)}",
            ]
        except Exception:
            return []

    # ── tool handlers ─────────────────────────────────────────────────────

    def handle_recall(self, ctx: Any, args: dict[str, Any]) -> Any:
        """Handler for noesis_recall tool."""
        query = args.get("query", "")
        logger.info("noesis_recall: query=%s", query[:60])
        results = self.recall(query)
        return json.dumps(results, indent=2) if results else "No results from Noesis."

    def handle_ingest(self, ctx: Any, args: dict[str, Any]) -> Any:
        """Handler for noesis_ingest tool."""
        text = args.get("text", "")
        if not text.strip():
            return tool_error("text is required")
        try:
            result = noesis.ingest(text.strip(), source="hermes")
            logger.info("noesis_ingest: %d chars -> %s", len(text), result.get("status"))
            return f"Recorded into Noesis: {result.get('status', 'accepted')} ({len(text)} chars)"
        except RuntimeError as e:
            logger.error("noesis_ingest failed: %s", e)
            return tool_error(str(e))

    def handle_status(self, ctx: Any, args: dict[str, Any]) -> Any:
        """Handler for noesis_status tool."""
        try:
            obs = noesis.observability()
            return json.dumps({
                "service": obs.get("service", "noesis"),
                "status": obs.get("status", "ok"),
                "uptime_seconds": obs.get("uptime_seconds", 0),
                "fields": obs.get("fields", 0),
                "processors": obs.get("processors", 0),
                "signal_types": obs.get("signal_types", 0),
                "signals_processed": obs.get("signals_processed", {}),
            }, indent=2)
        except RuntimeError as e:
            return tool_error(str(e))


# ---------------------------------------------------------------------------
# Entry point — called by Hermes Agent plugin loader
# ---------------------------------------------------------------------------


def register(ctx: Any) -> None:
    """Called by Hermes Agent to register the memory provider.

    Usage in hermes config:
        plugins:
          - name: noesis
            path: ~/noesis/hermes_integration
    """
    provider = NoesisProvider()
    provider.initialize(ctx)
    provider.register_tools(ctx)
    ctx.register_memory_provider(provider)
    logger.info("Noesis Hermes plugin registered")
