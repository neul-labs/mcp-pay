use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Payment rail configuration.
///
/// Each rail represents a different payment method the service accepts.
/// Rails are prioritized by the `priority` field (lower = preferred).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PaymentRail {
    /// Payment protocol type
    pub rail: RailType,

    /// Network identifier (CAIP-2 for chains, or custom network name)
    /// Examples: "eip155:8453" (Base), "solana:mainnet", "tempo", "bitcoin"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,

    /// Asset/token for payment
    /// Examples: "USDC", "USDT", "ETH", "SOL"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,

    /// Token contract address (for on-chain assets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract: Option<String>,

    /// Recipient address/account
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pay_to: Option<String>,

    /// Facilitator endpoint (for x402/MPP)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facilitator: Option<String>,

    /// Payment provider (for off-chain rails like cards)
    /// Examples: "stripe", "square", "coinbase"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// LNURL for Lightning payments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lnurl: Option<String>,

    /// Checkout URL for card payments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_url: Option<String>,

    /// Rail preference (lower = preferred)
    #[serde(default)]
    pub priority: u8,
}

impl Default for PaymentRail {
    fn default() -> Self {
        Self {
            rail: RailType::X402,
            network: None,
            asset: None,
            contract: None,
            pay_to: None,
            facilitator: None,
            provider: None,
            lnurl: None,
            checkout_url: None,
            priority: 0,
        }
    }
}

impl PaymentRail {
    /// Create an x402 payment rail on Base with USDC
    pub fn x402_base_usdc(pay_to: impl Into<String>) -> Self {
        Self {
            rail: RailType::X402,
            network: Some("eip155:8453".into()),
            asset: Some("USDC".into()),
            contract: Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".into()),
            pay_to: Some(pay_to.into()),
            facilitator: Some("https://x402.org/facilitator".into()),
            priority: 0,
            ..Default::default()
        }
    }

    /// Create an MPP payment rail on Tempo
    pub fn mpp_tempo(pay_to: impl Into<String>) -> Self {
        Self {
            rail: RailType::Mpp,
            network: Some("tempo".into()),
            asset: Some("USDC".into()),
            pay_to: Some(pay_to.into()),
            priority: 1,
            ..Default::default()
        }
    }

    /// Create a Lightning Network payment rail
    pub fn lightning(lnurl: impl Into<String>) -> Self {
        Self {
            rail: RailType::Lightning,
            network: Some("bitcoin".into()),
            lnurl: Some(lnurl.into()),
            priority: 2,
            ..Default::default()
        }
    }

    /// Create a card payment rail via Stripe
    pub fn stripe(checkout_url: impl Into<String>) -> Self {
        Self {
            rail: RailType::Card,
            provider: Some("stripe".into()),
            checkout_url: Some(checkout_url.into()),
            priority: 3,
            ..Default::default()
        }
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set the facilitator URL
    pub fn with_facilitator(mut self, url: impl Into<String>) -> Self {
        self.facilitator = Some(url.into());
        self
    }
}

/// Payment rail type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RailType {
    /// x402 protocol (HTTP 402 with on-chain settlement)
    X402,
    /// Machine Payments Protocol (Tempo)
    Mpp,
    /// Bitcoin Lightning Network
    Lightning,
    /// Traditional card payments (Stripe, Square, etc.)
    Card,
    /// ACH bank transfer
    Ach,
    /// Custom/vendor-specific rail
    Custom,
}

impl std::fmt::Display for RailType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RailType::X402 => write!(f, "x402"),
            RailType::Mpp => write!(f, "mpp"),
            RailType::Lightning => write!(f, "lightning"),
            RailType::Card => write!(f, "card"),
            RailType::Ach => write!(f, "ach"),
            RailType::Custom => write!(f, "custom"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x402_rail() {
        let rail = PaymentRail::x402_base_usdc("0x1234567890abcdef");
        let json = serde_json::to_string(&rail).unwrap();
        assert!(json.contains("\"rail\":\"x402\""));
        assert!(json.contains("\"network\":\"eip155:8453\""));
        assert!(json.contains("\"asset\":\"USDC\""));
    }

    #[test]
    fn test_lightning_rail() {
        let rail = PaymentRail::lightning("lnurl1dp68gurn8ghj7...");
        assert_eq!(rail.rail, RailType::Lightning);
        assert!(rail.lnurl.is_some());
    }

    #[test]
    fn test_rail_priority_sorting() {
        let mut rails = vec![
            PaymentRail::stripe("https://checkout.example.com"),
            PaymentRail::x402_base_usdc("0x123"),
            PaymentRail::lightning("lnurl..."),
        ];

        rails.sort_by_key(|r| r.priority);

        assert_eq!(rails[0].rail, RailType::X402);
        assert_eq!(rails[1].rail, RailType::Lightning);
        assert_eq!(rails[2].rail, RailType::Card);
    }
}
