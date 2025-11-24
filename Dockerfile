# Optimized Dockerfile using pre-compiled binaries
# The binaries are built by GitHub Actions and passed to this image
# This reduces build time from 20-30 minutes to < 1 minute

FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

# Copy pre-compiled binaries from GitHub Actions workflow
# The binaries are built with musl for static linking
COPY mcp-server-wazuh /app/mcp-server-wazuh
COPY mcp-http-server /app/mcp-http-server

# Distroless images:
# - Run as non-root user (uid 65532) by default
# - No shell, no package manager
# - Minimal attack surface (~20MB vs ~100MB+ for debian-slim)
# - Only contains runtime dependencies

# Expose port for HTTP server mode
EXPOSE 3000

# Default to stdio mode (can be overridden in Kubernetes deployment)
ENTRYPOINT ["/app/mcp-server-wazuh"]
