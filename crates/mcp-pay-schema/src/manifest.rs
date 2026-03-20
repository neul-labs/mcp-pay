use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pricing::Pricing;
use crate::rails::PaymentRail;
use crate::sla::PaymentSla;
use crate::stats::PaymentStats;

/// Root MCP-Pay manifest structure.
///
/// This is served at `/.well-known/mcp-pay.json` and declares
/// payment capabilities for an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpPayManifest {
    /// JSON Schema URL for validation
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Schema version (e.g., "0.1")
    pub mcp_pay: String,

    /// Link to MCP Server Card for general metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_card: Option<String>,

    /// Pricing rules for tools, resources, and prompts
    pub pricing: Pricing,

    /// Accepted payment rails
    pub accepts: Vec<PaymentRail>,

    /// Payment-specific SLA guarantees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_sla: Option<PaymentSla>,

    /// Payment statistics for discovery ranking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<PaymentStats>,

    /// Vendor-specific extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

impl McpPayManifest {
    /// Create a new manifest with required fields
    pub fn new(version: impl Into<String>, pricing: Pricing, accepts: Vec<PaymentRail>) -> Self {
        Self {
            schema: Some("https://mcp-pay.io/schema/v0.1/pay.schema.json".into()),
            mcp_pay: version.into(),
            server_card: None,
            pricing,
            accepts,
            payment_sla: None,
            stats: None,
            extensions: None,
        }
    }

    /// Set the server card link
    pub fn with_server_card(mut self, url: impl Into<String>) -> Self {
        self.server_card = Some(url.into());
        self
    }

    /// Set payment SLA
    pub fn with_sla(mut self, sla: PaymentSla) -> Self {
        self.payment_sla = Some(sla);
        self
    }

    /// Set payment stats
    pub fn with_stats(mut self, stats: PaymentStats) -> Self {
        self.stats = Some(stats);
        self
    }
}

impl Default for McpPayManifest {
    fn default() -> Self {
        Self {
            schema: Some("https://mcp-pay.io/schema/v0.1/pay.schema.json".into()),
            mcp_pay: "0.1".into(),
            server_card: None,
            pricing: Pricing::default(),
            accepts: vec![],
            payment_sla: None,
            stats: None,
            extensions: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rails::RailType;

    #[test]
    fn test_manifest_serialization() {
        let manifest = McpPayManifest::new(
            "0.1",
            Pricing::default(),
            vec![PaymentRail {
                rail: RailType::X402,
                network: Some("eip155:8453".into()),
                asset: Some("USDC".into()),
                ..Default::default()
            }],
        );

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("\"mcp_pay\": \"0.1\""));
        assert!(json.contains("\"rail\": \"x402\""));
    }

    #[test]
    fn test_manifest_deserialization() {
        let json = r#"{
            "mcp_pay": "0.1",
            "pricing": {},
            "accepts": []
        }"#;

        let manifest: McpPayManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.mcp_pay, "0.1");
    }
}
