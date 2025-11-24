# HTTP Server Guide

This guide explains how to use the HTTP wrapper server for the Wazuh MCP Server.

## Quick Start

### 1. Build the Project

```bash
cargo build --release
```

This will create two binaries:
- `target/release/mcp-server-wazuh` - Core MCP server (stdio)
- `target/release/mcp-http-server` - HTTP wrapper server

### 2. Configure Environment

Copy the example environment file and configure it:

```bash
cp .env.example .env
# Edit .env with your Wazuh credentials
```

Required variables:
- `WAZUH_API_HOST` - Your Wazuh Manager hostname
- `WAZUH_API_PORT` - Wazuh Manager API port (default: 55000)
- `WAZUH_API_USERNAME` - API username
- `WAZUH_API_PASSWORD` - API password
- `WAZUH_INDEXER_HOST` - Wazuh Indexer hostname
- `WAZUH_INDEXER_PORT` - Indexer port (default: 9200)
- `WAZUH_INDEXER_USERNAME` - Indexer username
- `WAZUH_INDEXER_PASSWORD` - Indexer password

### 3. Start the HTTP Server

```bash
# Default: listen on 0.0.0.0:3000
./target/release/mcp-http-server

# Custom host and port
./target/release/mcp-http-server --host 127.0.0.1 --port 8080

# Custom MCP binary location
./target/release/mcp-http-server --mcp-binary /path/to/mcp-server-wazuh
```

### 4. Test the Server

Run the test script:

```bash
./test-http-server.sh
```

Or manually test with curl:

```bash
# Health check
curl http://localhost:3000/health

# Initialize
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {"name": "test", "version": "1.0"}
    }
  }'

# List tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
  }'

# Call a tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "get_wazuh_agents",
      "arguments": {"limit": 5}
    }
  }'
```

## Available Endpoints

### GET /health

Health check endpoint. Returns "OK" if the server is running.

**Example:**
```bash
curl http://localhost:3000/health
```

**Response:**
```
OK
```

### POST /mcp

Main MCP endpoint. Accepts JSON-RPC 2.0 formatted requests and returns JSON-RPC 2.0 responses.

**Content-Type:** `application/json`

**Request Format:**
```json
{
  "jsonrpc": "2.0",
  "id": <number>,
  "method": "<method_name>",
  "params": {<parameters>}
}
```

**Response Format:**
```json
{
  "jsonrpc": "2.0",
  "id": <number>,
  "result": {<result_data>}
}
```

## Available Tools

All MCP tools are available via the HTTP endpoint. Common tools include:

- `get_wazuh_alert_summary` - Get security alerts
- `get_wazuh_agents` - List Wazuh agents
- `get_wazuh_rules_summary` - Get security rules
- `get_wazuh_vulnerability_summary` - Get vulnerabilities for an agent
- `get_wazuh_critical_vulnerabilities` - Get critical vulnerabilities
- `get_wazuh_agent_processes` - Get running processes on an agent
- `get_wazuh_agent_ports` - Get open ports on an agent
- `search_wazuh_manager_logs` - Search manager logs
- `get_wazuh_cluster_health` - Check cluster health
- `get_wazuh_cluster_nodes` - List cluster nodes

Use the `tools/list` method to get the complete list with descriptions and parameters.

## Architecture

```
┌─────────────────┐
│   HTTP Client   │
└────────┬────────┘
         │ POST /mcp
         │ (JSON-RPC 2.0)
         ▼
┌─────────────────────┐
│  mcp-http-server    │
│  (axum web server)  │
└─────────┬───────────┘
          │ stdin/stdout
          │ (JSON-RPC 2.0)
          ▼
┌─────────────────────┐
│ mcp-server-wazuh    │
│  (stdio MCP server) │
└─────────┬───────────┘
          │
          │ HTTPS API calls
          ▼
┌─────────────────────┐
│  Wazuh Manager +    │
│  Wazuh Indexer      │
└─────────────────────┘
```

## Features

- **CORS Enabled**: The server has permissive CORS enabled for development
- **Request Tracing**: All requests are logged with INFO level
- **Error Handling**: Proper JSON-RPC error responses
- **Process Management**: Automatically manages the MCP subprocess lifecycle
- **Concurrent Requests**: Supports multiple concurrent requests via tokio async

## Security Considerations

⚠️ **Important for Production:**

1. **Authentication**: The HTTP server does not implement authentication. Add a reverse proxy (nginx, Caddy) with auth if exposing publicly.

2. **CORS**: Currently uses permissive CORS. Restrict origins in production:
   ```rust
   // In src/bin/http-server.rs
   .layer(CorsLayer::new()
       .allow_origin("https://yourdomain.com".parse::<HeaderValue>().unwrap())
       .allow_methods([Method::GET, Method::POST])
   )
   ```

3. **SSL/TLS**: Use a reverse proxy with SSL termination (nginx, Caddy, Traefik)

4. **Rate Limiting**: Consider adding rate limiting middleware

5. **Firewall**: Restrict access to trusted IPs only

## Troubleshooting

### "Failed to spawn MCP server"

Ensure the MCP binary exists and is executable:
```bash
ls -l target/release/mcp-server-wazuh
chmod +x target/release/mcp-server-wazuh
```

Or specify the correct path:
```bash
./target/release/mcp-http-server --mcp-binary /full/path/to/mcp-server-wazuh
```

### "Connection refused"

Check if the server is running and on the correct port:
```bash
netstat -tlnp | grep 3000
# or
lsof -i :3000
```

### "Invalid JSON" or "MCP communication error"

Enable debug logging to see the communication:
```bash
RUST_LOG=debug ./target/release/mcp-http-server
```

Check the MCP server's stderr output (it's piped through) for any errors.

### Environment Variables Not Working

Ensure your `.env` file is in the same directory where you run the HTTP server, or export them:
```bash
export WAZUH_API_HOST=your_host
export WAZUH_API_USERNAME=your_user
# ... other vars
./target/release/mcp-http-server
```

## Development

To run the HTTP server in development mode with hot reload:

```bash
cargo watch -x 'run --bin mcp-http-server'
```

Enable debug logging:
```bash
RUST_LOG=debug cargo run --bin mcp-http-server
```

## Example Integration

### Python Client

```python
import requests
import json

MCP_URL = "http://localhost:3000/mcp"

def call_mcp_tool(tool_name, arguments):
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    }

    response = requests.post(MCP_URL, json=payload)
    return response.json()

# Get alerts
result = call_mcp_tool("get_wazuh_alert_summary", {"limit": 10})
print(json.dumps(result, indent=2))
```

### JavaScript Client

```javascript
const MCP_URL = "http://localhost:3000/mcp";

async function callMcpTool(toolName, args) {
  const response = await fetch(MCP_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "tools/call",
      params: {
        name: toolName,
        arguments: args,
      },
    }),
  });

  return response.json();
}

// Get agents
const result = await callMcpTool("get_wazuh_agents", { limit: 5 });
console.log(result);
```

## Performance Notes

- Each HTTP request spawns a single subprocess if not already running
- The subprocess is kept alive across requests (reused)
- Response times depend on Wazuh API latency
- Typical request: 100-500ms (depending on Wazuh query complexity)

## Logging

The HTTP server logs to stdout. The MCP subprocess logs to stderr (visible in the terminal).

Log levels:
- `INFO` (default): Request/response info
- `DEBUG`: Detailed communication traces
- `TRACE`: Very verbose, all internal operations

Set via `RUST_LOG` environment variable:
```bash
RUST_LOG=debug ./target/release/mcp-http-server
```
