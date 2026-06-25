#!/usr/bin/env python3
"""Noesis MCP Server — exposes the Noesis cognitive architecture over MCP.

Tools:
  noesis_status     — System health, stats, and observability
  noesis_ingest     — Inject raw text into the cognition pipeline
  noesis_recall     — Semantic retrieval over memory + knowledge graph
  noesis_graph      — Query the knowledge graph
  noesis_signals    — Signal processing metrics and statistics

Transport:
  streamable-http   — Long-lived service on :8645 (default)
  stdio             — Per-client (for Claude Code, etc.)

Run: python -m mcp_server.server
"""
from __future__ import annotations

import json
import os
import sys
from typing import Any

from mcp.server.fastmcp import FastMCP

from mcp_server import client as noesis

# ── config ──────────────────────────────────────────────────────────────

HOST = os.environ.get("NOESIS_MCP_HOST", "127.0.0.1")
PORT = int(os.environ.get("NOESIS_MCP_PORT", "8645"))
TRANSPORT = os.environ.get("NOESIS_MCP_TRANSPORT", "streamable-http")

# ── MCP server ──────────────────────────────────────────────────────────

mcp = FastMCP("Noesis", host=HOST, port=PORT)


# ── tools ────────────────────────────────────────────────────────────────


@mcp.tool()
def noesis_status() -> str:
    """Get the current status of the Noesis cognitive system.

    Returns health, uptime, field/processor/signal counts,
    and signal processing metrics.
    Useful for checking if the system is running and healthy.
    """
    try:
        obs = noesis.observability()
        return json.dumps({
            "service": obs.get("service", "noesis"),
            "version": obs.get("version", "?"),
            "uptime_seconds": obs.get("uptime_seconds", 0),
            "fields": obs.get("fields", 0),
            "processors": obs.get("processors", 0),
            "signal_types": obs.get("signal_types", 0),
            "signals_processed": obs.get("signals_processed", {}),
        }, indent=2)
    except RuntimeError as e:
        return f"Error: {e}"


@mcp.tool()
def noesis_ingest(text: str) -> str:
    """Record a new experience into the Noesis cognitive architecture.

    The text is injected into the signal cascade — it will be recorded as
    an episode, extracted into triples for the knowledge graph, and may
    trigger consolidation, attention shifts, belief changes, and narrative.

    Args:
        text: The experience text to record.
    """
    if not text.strip():
        return "Error: text is required"
    try:
        result = noesis.ingest(text.strip())
        return f"Recorded into Noesis: {result.get('status', 'accepted')} ({len(text)} chars)"
    except RuntimeError as e:
        return f"Error: {e}"


@mcp.tool()
def noesis_recall(query: str, k: int = 6) -> str:
    """Search memory and the knowledge graph in Noesis.

    Retrieves relevant context from both graph entities and stored episodes.
    Use for recalling facts, people, projects, concepts, and past experiences.

    Args:
        query: What to search for.
        k: Max results (default 6).
    """
    try:
        result = noesis.recall(query, k)
        if not result.get("results"):
            return f"No results found for: {query}"
        lines = [f"## Recall: {query} ({result['total']} results)\n"]
        for r in result["results"][:k]:
            score = r.get("score", 0)
            source = r.get("source", "?")
            text = r.get("text", "")
            score_bar = "█" * int(score * 10) + "░" * (10 - int(score * 10))
            lines.append(f"  [{score_bar}] ({source}) {text[:200]}")
        return "\n".join(lines)
    except RuntimeError as e:
        return f"Error: {e}"


@mcp.tool()
def noesis_graph(entity: str | None = None) -> str:
    """Query the Noesis knowledge graph.

    Without an entity, returns graph overview (entity/relation counts).
    With an entity name, returns details and connected relations.

    Args:
        entity: Optional entity name to expand.
    """
    try:
        from mcp_server.client import _api as noesis_api

        if entity:
            result = noesis_api("GET", f"/api/graph/expand?entity={entity}")
            ent = result.get("entity", {})
            name = ent.get("name", entity) if isinstance(ent, dict) else ent
            relations = result.get("connected_relations", [])
            count = result.get("relation_count", 0)
            lines = [f"## Entity: {name} ({count} relations)"]
            for r in relations[:10]:
                pred = r.get("predicate", "?")
                subj = r.get("subject_id", "?")[:8]
                obj = r.get("object_id", "?")[:8]
                conf = r.get("confidence", 0)
                lines.append(f"  {subj} --[{pred}]--> {obj} (confidence: {conf})")
            return "\n".join(lines)
        else:
            g = noesis.graph()
            return json.dumps({
                "entity_count": len(g.get("graph", {}).get("entities", [])),
                "relation_count": len(g.get("graph", {}).get("relations", [])),
            }, indent=2)
    except RuntimeError as e:
        return f"Error: {e}"


@mcp.tool()
def noesis_signals() -> str:
    """Get signal processing metrics from the Noesis cascade.

    Returns per-signal-type counts and processor dispatch statistics.
    Useful for understanding what the system is processing.
    """
    try:
        sig_stats = noesis.signal_stats()
        obs = noesis.observability()
        signals = sig_stats.get("signals", {})
        processors = obs.get("processor_stats", {})

        lines = ["## Signal Counts"]
        for name, count in sorted(signals.items(), key=lambda x: -x[1]):
            lines.append(f"  {name}: {count}x")

        lines.append("\n## Processor Metrics")
        for name, metrics in sorted(processors.items(), key=lambda x: -x[1].get("count", 0)):
            c = metrics.get("count", 0)
            ms = metrics.get("avg_latency_ms", 0)
            lines.append(f"  {name}: {c}x ({ms}ms avg)")

        return "\n".join(lines)
    except RuntimeError as e:
        return f"Error: {e}"


# ── entrypoint ──────────────────────────────────────────────────────────

if __name__ == "__main__":
    if TRANSPORT == "stdio":
        mcp.run(transport="stdio")
    else:
        print(f"Noesis MCP server starting on {HOST}:{PORT}...", file=sys.stderr)
        mcp.run(transport="sse")
