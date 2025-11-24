//! HTTP Server wrapper for the Wazuh MCP Server
//!
//! This binary launches the MCP server as a subprocess and exposes its functionality
//! via HTTP endpoints, allowing remote access to the MCP server's tools.
//!
//! Architecture:
//! - Spawns mcp-server-wazuh as a child process communicating via stdin/stdout
//! - Exposes POST /mcp endpoint that accepts JSON-RPC 2.0 requests
//! - Forwards requests to the MCP server's stdin
//! - Returns responses from the MCP server's stdout
//!
//! Usage:
//!   cargo run --bin http-server -- --port 3000 --host 0.0.0.0

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use clap::Parser;
use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Arc,
};
use tokio::sync::Mutex;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

#[derive(Parser, Debug)]
#[command(name = "mcp-http-server")]
#[command(about = "HTTP wrapper for Wazuh MCP Server")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Host address to bind to
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Path to the mcp-server-wazuh binary
    #[arg(long, default_value = "./target/release/mcp-server-wazuh")]
    mcp_binary: String,
}

/// Shared state holding the MCP server process
struct AppState {
    mcp_process: Arc<Mutex<McpProcess>>,
}

/// Wrapper around the MCP server child process
struct McpProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl McpProcess {
    /// Start a new MCP server process
    fn start(binary_path: &str) -> anyhow::Result<Self> {
        tracing::info!("Starting MCP server process: {}", binary_path);

        let mut child = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Keep stderr for logging
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn MCP server: {}. Make sure to build with 'cargo build --release' first", e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to open stdout"))?;

        let stdout = BufReader::new(stdout);

        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }

    /// Send a JSON-RPC request and read the response
    fn send_request(&mut self, request: &Value) -> anyhow::Result<Value> {
        // Write request to stdin
        let request_str = serde_json::to_string(request)?;
        tracing::debug!("Sending to MCP: {}", request_str);

        writeln!(self.stdin, "{}", request_str)?;
        self.stdin.flush()?;

        // Read response from stdout
        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line)?;

        tracing::debug!("Received from MCP: {}", response_line);

        let response: Value = serde_json::from_str(&response_line)?;
        Ok(response)
    }
}

impl Drop for McpProcess {
    fn drop(&mut self) {
        tracing::info!("Terminating MCP server process");
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Health check endpoint
async fn health() -> &'static str {
    "OK"
}

/// Main MCP endpoint that forwards JSON-RPC requests
async fn mcp_handler(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<Json<Value>, AppError> {
    let request: Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON: {}", e)))?;

    let mut process = state.mcp_process.lock().await;
    let response = process
        .send_request(&request)
        .map_err(|e| AppError::Internal(format!("MCP communication error: {}", e)))?;

    Ok(Json(response))
}

/// Application error type
#[derive(Debug)]
enum AppError {
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = serde_json::json!({
            "error": message
        });

        (status, Json(body)).into_response()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(Level::INFO.into()),
        )
        .init();

    tracing::info!("🚀 Starting Wazuh MCP HTTP Server");
    tracing::info!("📍 Binding to {}:{}", args.host, args.port);

    // Start the MCP server process
    let mcp_process = McpProcess::start(&args.mcp_binary)?;

    let state = Arc::new(AppState {
        mcp_process: Arc::new(Mutex::new(mcp_process)),
    });

    // Build the router
    let app = Router::new()
        .route("/health", get(health))
        .route("/mcp", post(mcp_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Bind and serve
    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("✅ HTTP server listening on http://{}", addr);
    tracing::info!("📡 MCP endpoint: POST http://{}/mcp", addr);
    tracing::info!("🏥 Health check: GET http://{}/health", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
