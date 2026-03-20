//! MCP-Pay Schema Library
//!
//! This crate provides types and validation for `mcp-pay.json` manifests,
//! which declare payment capabilities for MCP servers.
//!
//! # Overview
//!
//! The `mcp-pay.json` manifest is served at `/.well-known/mcp-pay.json` and
//! enables agents to discover:
//!
//! - **Pricing**: How much each tool/resource costs
//! - **Payment Rails**: Which payment methods are accepted (x402, MPP, card, etc.)
//! - **Payment SLA**: Settlement times, refund policies
//! - **Stats**: Transaction volume, success rates
//!
//! # Example
//!
//! ```rust
//! use mcp_pay_schema::{
//!     McpPayManifest, Pricing, PricingRule, PaymentRail,
//! };
//!
//! // Create a manifest for a paid weather API
//! let mut pricing = Pricing::with_default(PricingRule::free());
//! pricing.add_tool("get_forecast", PricingRule::per_call("0.003", "USD"));
//!
//! let manifest = McpPayManifest::new(
//!     "0.1",
//!     pricing,
//!     vec![PaymentRail::x402_base_usdc("0x1234...")],
//! );
//!
//! // Serialize to JSON
//! let json = serde_json::to_string_pretty(&manifest).unwrap();
//! println!("{}", json);
//! ```
//!
//! # Schema Relationship
//!
//! `mcp-pay.json` complements the MCP Server Card (SEP-1649):
//!
//! - **Server Card**: General metadata (name, capabilities, auth)
//! - **mcp-pay.json**: Payment-specific information
//!
//! Use the `server_card` field to link to the Server Card for general metadata.

mod manifest;
mod pricing;
mod rails;
mod sla;
mod stats;
mod validation;

pub use manifest::McpPayManifest;
pub use pricing::{
    BillingUnit, FreeQuota, Pricing, PricingModel, PricingRule, PricingTier, QuotaPeriod,
    TierLimit,
};
pub use rails::{PaymentRail, RailType};
pub use sla::{PaymentSla, RefundPolicy};
pub use stats::PaymentStats;
pub use validation::{validate_manifest, validate_with_schema, ValidationError};

/// Current schema version
pub const SCHEMA_VERSION: &str = "0.1";

/// Schema URL
pub const SCHEMA_URL: &str = "https://mcp-pay.io/schema/v0.1/mcp-pay.schema.json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_manifest_roundtrip() {
        // Create a complete manifest
        let mut pricing = Pricing::with_default(PricingRule::free());
        pricing.add_tool("get_forecast", PricingRule::per_call("0.003", "USD"));
        pricing.add_tool(
            "get_historical",
            PricingRule::tiered(
                "USD",
                vec![
                    PricingTier::new(1000u64, "0.01"),
                    PricingTier::new(10000u64, "0.008"),
                    PricingTier::new("unlimited", "0.005"),
                ],
            ),
        );

        let manifest = McpPayManifest::new(
            "0.1",
            pricing,
            vec![
                PaymentRail::x402_base_usdc("0x1234567890abcdef"),
                PaymentRail::lightning("lnurl1dp68gurn8ghj7..."),
            ],
        )
        .with_server_card("/.well-known/mcp/server-card.json")
        .with_sla(PaymentSla::crypto_default())
        .with_stats(
            PaymentStats::with_transactions(15420000, "46260.00", 2341)
                .with_success_rate("99.8")
                .with_avg_settlement(1200),
        );

        // Serialize
        let json = serde_json::to_string_pretty(&manifest).unwrap();

        // Deserialize
        let parsed: McpPayManifest = serde_json::from_str(&json).unwrap();

        // Verify
        assert_eq!(parsed.mcp_pay, "0.1");
        assert_eq!(parsed.accepts.len(), 2);
        assert!(parsed.payment_sla.is_some());
        assert!(parsed.stats.is_some());
    }

    #[test]
    fn test_minimal_manifest() {
        let json = r#"{
            "mcp_pay": "0.1",
            "pricing": {},
            "accepts": []
        }"#;

        let manifest: McpPayManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.mcp_pay, "0.1");
        assert!(manifest.accepts.is_empty());
    }
}
