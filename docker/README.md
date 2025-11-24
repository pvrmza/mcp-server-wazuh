# Docker Deployment for MCP Server Wazuh

This directory contains Docker Compose configurations for deploying the MCP Server Wazuh in different modes.

## Services

The docker-compose.yml file provides two service configurations:

### 1. `mcp-server-stdio`
- **Purpose**: Runs the MCP server in stdio mode for local MCP clients
- **Use case**: Testing, development, or when the server will be spawned as a subprocess
- **Ports**: No ports exposed (uses stdin/stdout communication)

### 2. `mcp-server-http`
- **Purpose**: Runs the MCP server in HTTP mode for remote access
- **Use case**: Production deployments, web applications, API integrations
- **Ports**: Exposes port 3000 (configurable via `MCP_HTTP_PORT`)
- **Endpoints**:
  - `GET /health` - Health check endpoint
  - `POST /mcp` - Main MCP endpoint (JSON-RPC 2.0)

## Quick Start

### 1. Configuration

Create your environment configuration file:

```bash
cp .env.example .env
```

Edit `.env` with your Wazuh configuration:

```bash
# Wazuh Manager API
WAZUH_API_HOST=your-wazuh-manager-host
WAZUH_API_PORT=55000
WAZUH_API_USERNAME=your-username
WAZUH_API_PASSWORD=your-password

# Wazuh Indexer API
WAZUH_INDEXER_HOST=your-wazuh-indexer-host
WAZUH_INDEXER_PORT=9200
WAZUH_INDEXER_USERNAME=admin
WAZUH_INDEXER_PASSWORD=admin

# SSL Configuration (set to true for production)
WAZUH_VERIFY_SSL=false

# Logging
RUST_LOG=info
```

### 2. Run HTTP Server (Recommended)

For remote access and web applications:

```bash
docker-compose up -d mcp-server-http
```

### 3. Run Stdio Server

For testing or development:

```bash
docker-compose up mcp-server-stdio
```

## Testing the HTTP Server

### Health Check

```bash
curl http://localhost:3000/health
```

Expected response:
```json
{"status":"healthy"}
```

### List Available Tools

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

### Get Alert Summary

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "get_wazuh_alert_summary",
      "arguments": {"limit": 10}
    }
  }'
```

## Building from Source

If you prefer to build the image locally instead of using the pre-built image:

1. Build the binaries first:

```bash
cd ..
cargo build --release
```

2. Uncomment the `build` section and update the command in `docker-compose.yml`:

```yaml
services:
  mcp-server-http:
    # image: ghcr.io/pvrmza/mcp-server-wazuh:latest
    build:
      context: ..
      dockerfile: Dockerfile.dev  # You'll need to create this
    command: ["--port", "3000", "--host", "0.0.0.0", "--mcp-binary", "/app/mcp-server-wazuh"]
    # ... rest of configuration
```

3. Create a `Dockerfile.dev` for local builds:

```dockerfile
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries from local build
COPY target/release/mcp-server-wazuh /app/mcp-server-wazuh
COPY target/release/mcp-http-server /app/mcp-http-server

RUN chmod +x /app/mcp-server-wazuh /app/mcp-http-server

EXPOSE 3000

ENTRYPOINT ["/app/mcp-server-wazuh"]
```

2. Build and run:

```bash
docker-compose build
docker-compose up -d mcp-server-http
```

## Production Deployment

For production environments, consider these security best practices:

1. **Enable SSL Verification**:
   ```bash
   WAZUH_VERIFY_SSL=true
   ```

2. **Use Proper SSL Certificates**: Ensure your Wazuh deployment has valid SSL certificates

3. **Use Docker Secrets**: For sensitive credentials, consider using Docker secrets instead of environment variables:

```yaml
services:
  mcp-server-http:
    secrets:
      - wazuh_api_password
      - wazuh_indexer_password
    environment:
      - WAZUH_API_PASSWORD_FILE=/run/secrets/wazuh_api_password
      - WAZUH_INDEXER_PASSWORD_FILE=/run/secrets/wazuh_indexer_password

secrets:
  wazuh_api_password:
    file: ./secrets/wazuh_api_password.txt
  wazuh_indexer_password:
    file: ./secrets/wazuh_indexer_password.txt
```

4. **Configure Logging**: Use appropriate log levels for production:
   ```bash
   RUST_LOG=info,mcp_server_wazuh=warn
   ```

5. **Network Security**: Consider using Docker networks to isolate the MCP server:

```yaml
networks:
  wazuh-network:
    external: true

services:
  mcp-server-http:
    networks:
      - wazuh-network
```

## Troubleshooting

### Check Container Logs

```bash
# HTTP server logs
docker-compose logs -f mcp-server-http

# Stdio server logs
docker-compose logs -f mcp-server-stdio
```

### Check Container Status

```bash
docker-compose ps
```

### Test Wazuh Connectivity

```bash
# Enter the container
docker-compose exec mcp-server-http sh

# Test Wazuh API connectivity (if shell is available)
# Note: Distroless images don't include shell by default
```

### Common Issues

1. **Connection Refused**: Check that `WAZUH_API_HOST` and `WAZUH_INDEXER_HOST` are accessible from within the container
2. **SSL Errors**: If using self-signed certificates, set `WAZUH_VERIFY_SSL=false`
3. **Authentication Errors**: Verify username and password are correct
4. **Port Conflicts**: If port 3000 is in use, change `MCP_HTTP_PORT` in `.env`

## Monitoring

The HTTP server includes a health check endpoint that Docker Compose uses automatically:

```yaml
healthcheck:
  test: ["CMD-SHELL", "wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 10s
```

Check health status:

```bash
docker inspect mcp-server-wazuh-http | grep -A 20 Health
```

## Cleanup

Stop and remove containers:

```bash
# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v

# Stop and remove images
docker-compose down --rmi all
```

## Additional Resources

- [MCP Server Wazuh Documentation](../README.md)
- [Claude Code Documentation](../CLAUDE.md)
- [Wazuh API Documentation](https://documentation.wazuh.com/current/user-manual/api/index.html)
