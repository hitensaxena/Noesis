# =============================================================================
# Noesis — Decentralized Cognitive Architecture
# Multi-stage Docker build: compile then distroless runtime
# =============================================================================

# ---- Stage 1: Build ----
FROM rust:1.96 AS builder
WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy real source and build
COPY src src/
COPY tests tests/
RUN cargo build --release --frozen

# ---- Stage 2: Runtime ----
FROM gcr.io/distroless/cc-debian12:latest

# Copy the compiled binary
COPY --from=builder /app/target/release/noesis /usr/local/bin/noesis

# Default plugin directory
RUN mkdir -p /root/.noesis/plugins

# Expose ports
# REST API
EXPOSE 8080
# MCP server
EXPOSE 8645

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD /usr/local/bin/noesis inspect health

ENTRYPOINT ["/usr/local/bin/noesis"]
CMD ["start", "--rest", "--port", "8080", "--mcp", "--mcp-port", "8645"]
