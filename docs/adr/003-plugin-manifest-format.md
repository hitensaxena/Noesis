# ADR-003: JSON-Manifest-Based Plugin Discovery

## Status

Accepted (2026-06-26)

## Context

Noesis needs a plugin system that supports dynamic loading of cognitive fields and processors. Plugins are shared libraries (`.dylib`/`.so`) loaded via `libloading`. The discovery mechanism must:

1. Find plugins installed in `~/.noesis/plugins/*/`
2. Identify plugin capabilities, processors, and signals without loading the library
3. Support metadata (name, version, author, description)

## Decision

We chose **JSON manifest files** (`plugin.json`) placed in each plugin directory:

```json
{
  "name": "my-plugin",
  "version": "0.1.0",
  "description": "Custom cognitive field",
  "author": "user",
  "library_path": "./libmy_plugin.dylib",
  "capabilities": [
    {
      "id": "custom.analyze",
      "name": "Custom Analysis",
      "description": "Performs custom cognitive analysis",
      "confidence": 0.8
    }
  ],
  "processors": ["custom_processor"],
  "signals": ["custom.signal.type"],
  "config_schema": {
    "type": "object",
    "properties": {
      "api_key": {"type": "string"}
    }
  }
}
```

## Rationale

- **No-load discovery** — manifest metadata is readable without loading the shared library, enabling fast `noesis plugins list`
- **Self-contained** — each plugin directory is a complete, movable unit
- **JSON standard** — no YAML parser dependency; serde already handles JSON
- **Extensible** — `capabilities` array and `config_schema` support plugin-specific configuration
- **Convention over configuration** — scan `~/.noesis/plugins/*/plugin.json` on startup

## Consequences

- Plugins must be compiled to native shared libraries matching the platform
- Dynamic loading uses `libloading` (Cargo feature: `dynamic-plugins`)
- Manifest validation happens at startup; invalid manifests are skipped with warnings
- Plugin-defined signal types must be registered before the kernel starts processing signals
- Security: plugins run as native code with full process access — users should only install trusted plugins
