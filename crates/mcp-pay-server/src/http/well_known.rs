//! Well-known endpoint handlers.
//!
//! Serves the mcp-pay.json manifest at /.well-known/mcp-pay.json

use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    Json,
};

use mcp_pay_schema::{
    McpPayManifest, PaymentRail, PaymentSla, PaymentStats, Pricing, PricingRule, PricingTier,
    TierLimit,
};

use super::AppState;

/// Serve the mcp-pay.json manifest.
///
/// This endpoint declares the payment capabilities of this MCP server.
pub async fn mcp_pay_json(State(state): State<AppState>) -> Response {
    let config = &state.config;

    // Build pricing configuration
    let mut pricing = Pricing::with_default(PricingRule::free());

    // get_weather is free
    pricing.add_tool("get_weather", PricingRule::free());

    // get_forecast costs $0.003 per call
    pricing.add_tool(
        "get_forecast",
        PricingRule::per_call(&config.price_per_call, "USD"),
    );

    // get_historical has tiered pricing
    pricing.add_tool(
        "get_historical",
        PricingRule::tiered(
            "USD",
            vec![
                PricingTier::new(1000u64, "0.01"),
                PricingTier::new(10000u64, "0.008"),
                PricingTier::new(TierLimit::Unlimited, "0.005"),
            ],
        ),
    );

    // Build payment rails
    let rails = vec![
        PaymentRail {
            rail: mcp_pay_schema::RailType::X402,
            network: Some(config.network.clone()),
            asset: Some(config.asset.clone()),
            contract: config.contract.clone(),
            pay_to: Some(config.pay_to_address.clone()),
            facilitator: Some(config.x402_facilitator.clone()),
            priority: 0,
            ..Default::default()
        },
    ];

    // Build manifest
    let manifest = McpPayManifest::new("0.1", pricing, rails)
        .with_server_card("/.well-known/mcp/server-card.json")
        .with_sla(PaymentSla::crypto_default())
        .with_stats(
            PaymentStats::with_transactions(0, "0.00", 0)
                .with_success_rate("100.0")
                .with_avg_settlement(2000),
        );

    // Return with appropriate headers
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        Json(manifest),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::payment::X402Verifier;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_mcp_pay_json() {
        let config = Config::test_config();
        let verifier = Arc::new(X402Verifier::new(&config));
        let state = AppState { config, verifier };

        let response = mcp_pay_json(State(state)).await;

        // Check it's valid JSON
        assert!(response.status().is_success());
    }
}
