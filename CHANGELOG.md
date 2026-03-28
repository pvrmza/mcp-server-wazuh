# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security
- **BREAKING CHANGE**: Credentials now required with no defaults
  - `WAZUH_API_USERNAME`, `WAZUH_API_PASSWORD` must be explicitly configured
  - `WAZUH_INDEXER_USERNAME`, `WAZUH_INDEXER_PASSWORD` must be explicitly configured
  - Server will fail to start if credentials are missing
  - **Migration Required**: Update your `.env` or environment configuration with actual credential values
- **Changed default**: `WAZUH_VERIFY_SSL` now defaults to `true` (was `false`)
  - Secure by default - production-ready SSL verification enabled out of the box
  - Only disable for development environments with self-signed certificates
  - **Migration Note**: If using self-signed certificates in development, explicitly set `WAZUH_VERIFY_SSL=false`

### Added
- **Flexible parameter types**: MCP tools now accept both String and Number for IDs and enum parameters
  - Improved compatibility with MCP clients that may convert parameter types automatically
  - Affected parameters: `agent_id`, `status`, `protocol`, `state`, `level`
  - Seamless type conversion using `deserialize_string_or_number` utility
  - Example: `agent_id` can be sent as `"001"` (string) or `1` (number) - both work

### Changed
- **Code refactoring**: Eliminated duplicate code across tool modules
  - Moved `deserialize_string_or_number` to `tools/mod.rs` (was duplicated in 3 files)
  - Consolidated `format_agent_id` usage in vulnerabilities module
  - Improved maintainability and reduced code complexity
  - No functional changes - internal refactoring only

### Dependencies
- **Upgraded `rmcp`** from 0.1.5 to 0.10
  - Adds support for Streamable HTTP transport with Server-Sent Events (SSE)
  - Updated to MCP protocol version `2025-06-18`
  - Enhanced session management capabilities
- **Upgraded `wazuh-client`** from 0.1.7 to 0.1.8
  - Improved unmarshaling for indexer responses
  - Fixed ordering issues in API responses
- **Upgraded `schemars`** from 0.8 to 1.0
  - Latest JSON Schema generation support
- **Added `axum`** 0.8 (optional, requires `http` feature)
  - Enables HTTP transport mode for remote server deployment

### Migration Guide

#### For Existing Users

**⚠️ Action Required - Breaking Changes:**

1. **Update Credentials** (Required):
   ```bash
   # In your .env file or environment:
   WAZUH_API_USERNAME=your_actual_username      # No default anymore
   WAZUH_API_PASSWORD=your_actual_password      # No default anymore
   WAZUH_INDEXER_USERNAME=your_actual_username  # No default anymore
   WAZUH_INDEXER_PASSWORD=your_actual_password  # No default anymore
   ```

2. **Review SSL Configuration** (Recommended):
   ```bash
   # Default is now true (secure)
   WAZUH_VERIFY_SSL=true

   # Only for development with self-signed certificates:
   WAZUH_VERIFY_SSL=false
   ```

3. **Test Before Deploying**:
   ```bash
   # Verify server starts with new configuration
   cargo run
   # Should fail if credentials are not set (expected behavior)
   ```

#### Compatibility Notes

- **No code changes required** for existing integrations
- **Configuration changes required** for environment variables
- Parameter flexibility improves compatibility with diverse MCP clients
- Recommended to update to latest `.env.example` as reference

## [0.3.1] - 2025-11-24

### Fixed
- **HTTP Server MCP Initialization**: Fixed critical issue where HTTP server did not automatically initialize MCP protocol connection
  - Added automatic `initialize` request on subprocess startup
  - Added automatic `notifications/initialized` notification handling
  - Fixed "expect initialize request" error that prevented all tool calls
  - Fixed "broken pipe" errors caused by subprocess crashes
- **JSON-RPC 2.0 Notifications**: Implemented proper handling of notifications (messages without `id` field)
  - HTTP server no longer waits for responses on notification messages
  - Fixed hanging requests when sending `notifications/initialized`

### Changed
- **HTTP Health Endpoint**: Now returns JSON response with service info instead of plain text
  - Response format: `{"status": "healthy", "service": "mcp-http-server", "version": "0.3.0"}`
- **Docker Configuration**: Updated docker-compose.yml with improved configuration
  - Added `--mcp-binary` flag to specify correct binary path in container
  - Updated environment variables to match current naming convention
  - Added comprehensive health check configuration
  - Separated stdio and HTTP service definitions

### Added
- **Docker Documentation**: Created comprehensive Docker deployment guide (`docker/README.md`)
  - Quick start instructions
  - Testing examples
  - Production deployment best practices
  - Troubleshooting guide
- **Docker Environment Configuration**: Added `docker/.env.example` with all required variables

### Documentation
- Updated `HTTP_SERVER.md` to reflect automatic initialization
- Simplified usage examples (no manual initialization required)
- Added notes about token management and JWT handling
- Updated feature list with new capabilities

## [0.3.0] - 2025-11-24

### Added
- **HTTP Server Mode**: New HTTP wrapper server (`mcp-http-server`) for remote access
  - Exposes MCP server via HTTP POST endpoint
  - Supports JSON-RPC 2.0 over HTTP
  - CORS enabled for web application integration
  - Health check endpoint at `/health`
  - Main MCP endpoint at `/mcp`

### Changed
- **Docker Image Optimization**: Switched to pre-compiled binary approach
  - Build time reduced from 20-30 minutes to < 1 minute
  - Uses distroless base image for minimal attack surface
  - GitHub Actions builds binaries before Docker image creation

### Documentation
- Added `HTTP_SERVER.md` with complete HTTP server guide
- Updated `README.md` with HTTP server installation and usage
- Updated `CLAUDE.md` with HTTP server architecture details

## [0.2.5] - 2025-11-21

### Changed
- **Docker Build Performance**: Optimized Dockerfile to use pre-compiled binaries
  - Significant reduction in build times
  - Smaller final image size

## Earlier Versions

See [GitHub Releases](https://github.com/pvrmza/mcp-server-wazuh/releases) for information about earlier versions.

---

## Version Support

- **0.3.x**: Current stable release with HTTP server support
- **0.2.x**: Stdio-only release (maintenance mode)

## Upgrade Guide

### From 0.3.0 to 0.3.1

No breaking changes. The HTTP server now works correctly out of the box:

1. Pull the new Docker image:
   ```bash
   docker pull ghcr.io/pvrmza/mcp-server-wazuh:latest
   ```

2. Restart your containers:
   ```bash
   docker compose down
   docker compose up -d
   ```

3. Test that it works:
   ```bash
   curl http://localhost:3000/health
   curl -X POST http://localhost:3000/mcp \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
   ```

### From 0.2.x to 0.3.x

The 0.3.x series adds HTTP server support but maintains backward compatibility with stdio mode:

- **Stdio mode**: No changes required, works as before
- **HTTP mode**: New feature, see `HTTP_SERVER.md` for setup instructions
- **Docker**: Updated compose file, review `docker/README.md` for changes

## Known Issues

### 0.3.1
- None currently reported

### 0.3.0
- ~~HTTP server requires manual MCP initialization~~ (Fixed in 0.3.1)
- ~~Notifications cause server to hang~~ (Fixed in 0.3.1)

## Contributing

When contributing, please:
1. Update this CHANGELOG.md with your changes
2. Follow [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format
3. Add entries under "Unreleased" section until the next release
