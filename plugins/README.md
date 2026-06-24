# Noesis Plugins

Plugins extend Noesis with new signals, processors, and fields without modifying the kernel.

## How Plugins Work

A plugin is a dynamic library (`.so` on Linux, `.dylib` on macOS) that exports a `NoesisPlugin` entry point.

Each plugin can register:
- New signal types
- New processors
- New fields
- Configuration schema
- Scheduled jobs

## Loading a Plugin

```bash
noesis plugins load ./path/to/plugin.so
noesis plugins list
```

## Creating a Plugin

TODO: Plugin API reference and example plugin.
