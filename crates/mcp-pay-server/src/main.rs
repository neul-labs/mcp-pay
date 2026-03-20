//! MCP-Pay Reference Server
//!
//! A reference implementation of an MCP server with payment gating.
//!
//! This server demonstrates:
//! - Serving `/.well-known/mcp-pay.json` for payment discovery
//! - HTTP 402 payment flow with x402
//! - Free and paid tool endpoints
//!
//! # Running
//!
//! ```bash
//! # Set environment variables (optional, has defaults)
//! export PAY_TO_ADDRESS=0x1234...
//! export HTTP_ADDR=127.0.0.1:3000
//!
//! # Run the server
//! cargo run -p mcp-pay-server
//! ```
//!
//! # Endpoints
//!
//! - `GET /.well-known/mcp-pay.json` - Payment manifest
//! - `GET /api/weather?city=London` - Current weather (FREE)
//! - `GET /api/forecast?city=London` - 7-day forecast (PAID)
//! - `GET /health` - Health check

mod config;
mod http;
mod payment;
mod tools;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use config::Config;
use http::create_router;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = Config::from_env()?;

    tracing::info!(
        service = %config.service_name,
        addr = %config.http_addr,
        network = %config.network,
        "Starting mcp-pay server"
    );

    // Create HTTP router
    let app = create_router(config.clone());

    // Bind and serve
    let listener = TcpListener::bind(&config.http_addr).await?;

    tracing::info!(
        "Server listening on http://{}",
        config.http_addr
    );
    tracing::info!(
        "Payment manifest at http://{}/.well-known/mcp/pay.json",
        config.http_addr
    );

    axum::serve(listener, app).await?;

    Ok(())
}
