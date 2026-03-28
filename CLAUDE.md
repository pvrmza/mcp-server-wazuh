# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based MCP (Model Context Protocol) server that bridges Wazuh SIEM (Security Information and Event Management) systems with AI assistants and automation tools. The server exposes Wazuh security data through MCP tools, enabling natural language interaction with security alerts, vulnerabilities, agent management, and compliance monitoring.

The server communicates via stdio using JSON-RPC 2.0 and is built on the `rmcp` framework (v0.10) with the `wazuh-client` crate (v0.1.8) for Wazuh API interactions.

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
| `WAZUH_API_USERNAME` | Wazuh Manager API username | **(required)** |
| `WAZUH_API_PASSWORD` | Wazuh Manager API password | **(required)** |
| `WAZUH_INDEXER_HOST` | Wazuh Indexer hostname/IP | `localhost` |
| `WAZUH_INDEXER_PORT` | Wazuh Indexer port | `9200` |
| `WAZUH_INDEXER_USERNAME` | Wazuh Indexer username | **(required)** |
| `WAZUH_INDEXER_PASSWORD` | Wazuh Indexer password | **(required)** |
| `WAZUH_VERIFY_SSL` | Enable SSL certificate verification | `true` |
| `WAZUH_TEST_PROTOCOL` | Protocol override (`http` or `https`) | `https` |
| `RUST_LOG` | Logging level (`info`, `debug`, `trace`) | `info` |

**Security Note**: `WAZUH_VERIFY_SSL` defaults to `true` (secure by default). All credential environment variables (`WAZUH_API_USERNAME`, `WAZUH_API_PASSWORD`, `WAZUH_INDEXER_USERNAME`, `WAZUH_INDEXER_PASSWORD`) are mandatory. The server will not start without properly configured credentials.

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
- `rmcp` (0.10): MCP server framework with stdio and HTTP transports
- `wazuh-client` (0.1.8): Wazuh API client library
- `tokio`: Async runtime with full features
- `reqwest`: HTTP client with rustls-tls
- `serde` / `serde_json`: Serialization
- `schemars` (1.0): JSON Schema generation for MCP tool parameters
- `anyhow` / `thiserror`: Error handling
- `tracing` / `tracing-subscriber`: Structured logging

### HTTP Server Dependencies
- `axum` (0.8, optional): Web framework for HTTP transport
- `tower`: Service middleware and utilities
- `tower-http`: HTTP-specific middleware (CORS, tracing)
- `bytes`: Byte buffer utilities

### Test Dependencies
- `httpmock`: HTTP mock server for integration tests
- `mockito`: Additional HTTP mocking
- `tokio-test`: Async test utilities

## Recent Improvements (v0.3.0+)

### Security Enhancements
- **Secure by Default**: `WAZUH_VERIFY_SSL` now defaults to `true` (was `false`)
- **Required Credentials**: All authentication variables are now mandatory with no default values:
  - `WAZUH_API_USERNAME`, `WAZUH_API_PASSWORD` (no longer defaults to "wazuh")
  - `WAZUH_INDEXER_USERNAME`, `WAZUH_INDEXER_PASSWORD` (no longer defaults to "admin")
  - Server fails fast on startup if credentials are not configured

### Flexible Parameter Types
- MCP tools now accept both String and Number types for better client compatibility
- Affected parameters: `agent_id`, `status`, `protocol`, `state`, `level`
- Automatic type conversion handled by `deserialize_string_or_number` utility

### Code Quality
- Eliminated duplicate code across tool modules
- Consolidated `deserialize_string_or_number` in `tools/mod.rs` (was duplicated 3x)
- Consistent use of `ToolUtils::format_agent_id()` across all modules
- Improved maintainability and consistency

### Transport Layer
- Upgraded to `rmcp` 0.10 with Streamable HTTP transport support
- Optional HTTP mode via `--transport http` flag (requires `http` feature)
- MCP protocol version: `2025-06-18`

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
4. **SSL Verification**: Defaults to `true` (secure). Only disable for development with self-signed certificates
5. **Required Credentials**: Server will fail to start if `WAZUH_API_USERNAME`, `WAZUH_API_PASSWORD`, `WAZUH_INDEXER_USERNAME`, or `WAZUH_INDEXER_PASSWORD` are not set
6. **Flexible Parameters**: MCP tools accept both String and Number types for parameters like `agent_id`, `status`, `protocol`, `state`, and `level` for better client compatibility
7. **Error Context**: Use `Self::format_error()` from `ToolModule` trait for consistent error messages

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
