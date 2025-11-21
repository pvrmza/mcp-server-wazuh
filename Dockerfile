# Optimized Dockerfile using pre-compiled binary
# The binary is built by GitHub Actions and passed to this image
# This reduces build time from 20-30 minutes to < 1 minute

FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

# Copy pre-compiled binary from GitHub Actions workflow
# The binary is built with musl for static linking
COPY mcp-server-wazuh /app/mcp-server-wazuh

# Distroless images:
# - Run as non-root user (uid 65532) by default
# - No shell, no package manager
# - Minimal attack surface (~20MB vs ~100MB+ for debian-slim)
# - Only contains runtime dependencies

# Optional: expose port for documentation
EXPOSE 8000

ENTRYPOINT ["/app/mcp-server-wazuh"]
