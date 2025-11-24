# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based MCP (Model Context Protocol) server that bridges Wazuh SIEM (Security Information and Event Management) systems with AI assistants and automation tools. The server exposes Wazuh security data through MCP tools, enabling natural language interaction with security alerts, vulnerabilities, agent management, and compliance monitoring.

The server communicates via stdio using JSON-RPC 2.0 and is built on the `rmcp` framework (v0.1.5) with the `wazuh-client` crate (v0.1.7) for Wazuh API interactions.

## Build and Development Commands

### Building
```bash
# Development build
cargo build

# Release build
cargo build --release
```

The release binary is located at `target/release/mcp-server-wazuh`.

### Running
```bash
# Run with cargo (uses .env file)
cargo run

# Run release binary directly (requires environment variables)
./target/release/mcp-server-wazuh
```

### HTTP Server Mode

The project includes an HTTP wrapper server (`mcp-http-server`) that exposes the MCP server via HTTP endpoints:

```bash
# Build both binaries
cargo build --release

# Run HTTP server
./target/release/mcp-http-server --port 3000 --host 0.0.0.0

# With custom MCP binary location
./target/release/mcp-http-server \
  --port 3000 \
  --host 0.0.0.0 \
  --mcp-binary ./target/release/mcp-server-wazuh
```

**Available Endpoints:**
- `GET /health` - Health check endpoint
- `POST /mcp` - Main MCP endpoint accepting JSON-RPC 2.0 requests

**Architecture:**
- HTTP server spawns the stdio MCP server as a child process
- Accepts HTTP POST requests with JSON-RPC 2.0 payloads
- Forwards requests to MCP server's stdin
- Returns responses from MCP server's stdout
- Enables remote access and web application integration

**Example HTTP Request:**
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

### Testing
```bash
# Run all tests (unit + integration)
cargo test

# Run with detailed logging
RUST_LOG=debug cargo test -- --nocapture

# Run specific test suites
cargo test --test rmcp_integration_test    # Integration tests with mock Wazuh
cargo test --test mcp_stdio_test           # MCP protocol compliance tests
cargo test --lib                           # Unit tests only
```

See `tests/README.md` for comprehensive testing documentation.

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run linter with all warnings as errors
cargo clippy -- -D warnings
```

### Docker
```bash
# Build image
docker build -t mcp-server-wazuh .

# Pull from GitHub Container Registry
docker pull ghcr.io/gbrigandi/mcp-server-wazuh:latest
```

## Architecture

### High-Level Structure

The application follows a **modular facade pattern** where `main.rs` contains the central `WazuhToolsServer` that delegates to domain-specific tool modules:

```
main.rs (WazuhToolsServer)
    ├── tools/alerts.rs (AlertTools)         → Wazuh Indexer API
    ├── tools/agents.rs (AgentTools)         → Wazuh Manager API
    ├── tools/rules.rs (RuleTools)           → Wazuh Manager API
    ├── tools/vulnerabilities.rs (VulnerabilityTools) → Wazuh Manager API
    └── tools/stats.rs (StatsTools)          → Wazuh Manager API
```

### Key Design Patterns

1. **Facade Pattern**: `WazuhToolsServer` acts as a unified interface that routes MCP tool calls to specialized domain modules
2. **Tool Modules**: Each domain module (`*Tools` structs) encapsulates:
   - Business logic for their Wazuh component
   - Parameter validation and error handling
   - Client interactions with specific Wazuh APIs
   - Output formatting (rich text with emojis)
3. **Client Management**: Wazuh clients are created by `WazuhClientFactory` and wrapped in `Arc<Mutex<>>` for thread-safe async access
4. **Separation of Concerns**: Tool-specific logic is isolated from MCP protocol handling

### MCP Tool Registration

Tools are registered using the `#[tool(...)]` attribute macro from the `rmcp` crate. The `WazuhToolsServer` struct is annotated with `#[tool(tool_box)]` and implements methods decorated with `#[tool(name = "...", description = "...")]`.

### Wazuh Client Interactions

The server interacts with two Wazuh components:
- **Wazuh Manager API** (port 55000): Agents, rules, vulnerabilities, logs, cluster, statistics
- **Wazuh Indexer API** (port 9200): Security alerts from Elasticsearch-compatible index

Clients are created in `WazuhToolsServer::new()` and distributed to tool modules.

### Agent ID Formatting

Agent IDs must be three-digit, zero-padded strings (e.g., "001", "012"). The `ToolUtils::format_agent_id()` utility in `tools/mod.rs` handles conversion from numeric or string inputs (e.g., "1" → "001", "12" → "012").

## Configuration

Configuration is managed through environment variables. For local development, copy `.env.example` to `.env`:

```bash
cp .env.example .env
```

### Required Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WAZUH_API_HOST` | Wazuh Manager API hostname/IP | `localhost` |
| `WAZUH_API_PORT` | Wazuh Manager API port | `55000` |
| `WAZUH_API_USERNAME` | Wazuh Manager API username | `wazuh` |
| `WAZUH_API_PASSWORD` | Wazuh Manager API password | `wazuh` |
| `WAZUH_INDEXER_HOST` | Wazuh Indexer hostname/IP | `localhost` |
| `WAZUH_INDEXER_PORT` | Wazuh Indexer port | `9200` |
| `WAZUH_INDEXER_USERNAME` | Wazuh Indexer username | `admin` |
| `WAZUH_INDEXER_PASSWORD` | Wazuh Indexer password | `admin` |
| `WAZUH_VERIFY_SSL` | Enable SSL certificate verification | `false` |
| `WAZUH_TEST_PROTOCOL` | Protocol override (`http` or `https`) | `https` |
| `RUST_LOG` | Logging level (`info`, `debug`, `trace`) | `info` |

**Security Note**: For production, always set `WAZUH_VERIFY_SSL=true` with proper certificates.

## Adding New MCP Tools

To add a new tool, follow this pattern:

1. **Create or extend a tool module** in `src/tools/`:
   ```rust
   // Define parameter struct with serde + schemars
   #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
   pub struct MyToolParams {
       #[schemars(description = "Parameter description")]
       pub my_param: Option<String>,
   }

   // Implement tool method in the appropriate *Tools struct
   impl MyTools {
       pub async fn my_tool(
           &self,
           params: MyToolParams,
       ) -> Result<CallToolResult, McpError> {
           // Implementation
       }
   }
   ```

2. **Register in `WazuhToolsServer`** (main.rs):
   ```rust
   #[tool(
       name = "my_tool",
       description = "Tool description"
   )]
   async fn my_tool(
       &self,
       #[tool(aggr)] params: MyToolParams,
   ) -> Result<CallToolResult, McpError> {
       self.my_tools.my_tool(params).await
   }
   ```

3. **Use the `ToolModule` trait** for consistent error/success handling:
   - `Self::success_result(vec![Content::text(output)])`
   - `Self::error_result(error_message)`
   - `Self::not_found_result("resource name")`

## Testing Strategy

The project uses a three-tier testing approach:

1. **Unit Tests**: Test individual components in isolation (in tool modules)
2. **Integration Tests**: Test full MCP flows with mock Wazuh API (`tests/rmcp_integration_test.rs`)
3. **Protocol Tests**: Verify MCP JSON-RPC 2.0 compliance (`tests/mcp_stdio_test.rs`)

All tests run without requiring a real Wazuh instance. The mock server (`tests/mock_wazuh_server.rs`) simulates both Wazuh Manager and Indexer APIs.

## Dependencies

### Core Dependencies
- `rmcp` (0.1.5): MCP server framework with stdio transport
- `wazuh-client` (0.1.7): Wazuh API client library
- `tokio`: Async runtime with full features
- `reqwest`: HTTP client with rustls-tls
- `serde` / `serde_json`: Serialization
- `schemars`: JSON Schema generation for MCP tool parameters
- `anyhow` / `thiserror`: Error handling
- `tracing` / `tracing-subscriber`: Structured logging

### HTTP Server Dependencies
- `axum` (0.7): Web framework for HTTP server
- `tower`: Service middleware and utilities
- `tower-http`: HTTP-specific middleware (CORS, tracing)
- `bytes`: Byte buffer utilities

### Test Dependencies
- `httpmock`: HTTP mock server for integration tests
- `mockito`: Additional HTTP mocking
- `tokio-test`: Async test utilities

## Logging

Logs are written to stderr (stdout is reserved for MCP JSON-RPC communication). Configure via `RUST_LOG`:

```bash
# Basic logging
RUST_LOG=info cargo run

# Debug MCP server only
RUST_LOG=mcp_server_wazuh=debug cargo run

# Trace everything
RUST_LOG=trace cargo run

# Selective logging
RUST_LOG=info,mcp_server_wazuh=debug,wazuh_client=info cargo run
```

## MCP Protocol Notes

- **Transport**:
  - **Stdio** (default): JSON-RPC 2.0 over stdin/stdout for local MCP clients
  - **HTTP** (via wrapper): JSON-RPC 2.0 over HTTP POST for remote/web access
- **Protocol Version**: `2024-11-05`
- **Capabilities**: Tools, prompts, resources
- **Tool Results**: Use `CallToolResult::success()` or `CallToolResult::error()` with `Content::text()` items

The server implements the `ServerHandler` trait from `rmcp` and provides server info through the `get_info()` method.

### Binary Targets

The project provides two binary targets:
1. **`mcp-server-wazuh`** (src/main.rs): Core MCP server with stdio transport
2. **`mcp-http-server`** (src/bin/http-server.rs): HTTP wrapper server that launches the core MCP server as a subprocess

## Common Pitfalls

1. **Agent ID Format**: Always use `ToolUtils::format_agent_id()` before passing agent IDs to Wazuh clients
2. **Arc<Mutex<>> Locking**: Remember to `.lock().await` when accessing shared Wazuh clients
3. **Stdout Usage**: Never use stdout for logging or debugging; use `tracing::info!()` which logs to stderr
4. **SSL Verification**: Development often uses `WAZUH_VERIFY_SSL=false`; remember to enable for production
5. **Error Context**: Use `Self::format_error()` from `ToolModule` trait for consistent error messages

## Client Applications

This server is designed to integrate with:

### Stdio Clients
- **Claude Desktop** (via `claude_desktop_config.json`)
- **IDE extensions** supporting MCP
- **Custom automation tools** using MCP protocol

These clients launch the server as a subprocess and communicate via stdin/stdout.

### HTTP Clients
- **Web applications** making HTTP POST requests
- **Remote MCP clients** accessing the server over network
- **API integrations** using JSON-RPC 2.0 over HTTP
- **Curl/Postman/HTTPie** for testing and development

HTTP clients connect to `mcp-http-server` which proxies requests to the underlying MCP server.
