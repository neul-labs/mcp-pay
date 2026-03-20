use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pricing configuration for the service.
///
/// Pricing can be specified at multiple levels:
/// - `default`: applies to all tools/resources unless overridden
/// - `tools`: per-tool pricing overrides
/// - `resources`: per-resource pricing (supports glob patterns)
/// - `prompts`: per-prompt pricing overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Pricing {
    /// Default pricing for all capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<PricingRule>,

    /// Per-tool pricing overrides (tool name -> pricing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<HashMap<String, PricingRule>>,

    /// Per-resource pricing (resource URI pattern -> pricing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<HashMap<String, PricingRule>>,

    /// Per-prompt pricing overrides (prompt name -> pricing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<HashMap<String, PricingRule>>,
}

impl Pricing {
    /// Create pricing with a default rule
    pub fn with_default(rule: PricingRule) -> Self {
        Self {
            default: Some(rule),
            ..Default::default()
        }
    }

    /// Add tool pricing
    pub fn add_tool(&mut self, name: impl Into<String>, rule: PricingRule) {
        self.tools
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), rule);
    }

    /// Add resource pricing
    pub fn add_resource(&mut self, pattern: impl Into<String>, rule: PricingRule) {
        self.resources
            .get_or_insert_with(HashMap::new)
            .insert(pattern.into(), rule);
    }

    /// Get pricing for a specific tool
    pub fn get_tool_pricing(&self, tool_name: &str) -> Option<&PricingRule> {
        self.tools
            .as_ref()
            .and_then(|t| t.get(tool_name))
            .or(self.default.as_ref())
    }
}

/// Pricing rule for a capability.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PricingRule {
    /// Pricing model type
    pub model: PricingModel,

    /// Price amount as decimal string (e.g., "0.003")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// Currency code (ISO 4217 or token symbol)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Billing unit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<BillingUnit>,

    /// Tiered pricing breakpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiers: Option<Vec<PricingTier>>,

    /// Free quota allowance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_quota: Option<FreeQuota>,

    /// Subscription plan ID (for subscription model)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
}

impl PricingRule {
    /// Create a free pricing rule
    pub fn free() -> Self {
        Self {
            model: PricingModel::Free,
            amount: None,
            currency: None,
            unit: None,
            tiers: None,
            free_quota: None,
            subscription_id: None,
        }
    }

    /// Create a per-call pricing rule
    pub fn per_call(amount: impl Into<String>, currency: impl Into<String>) -> Self {
        Self {
            model: PricingModel::PerCall,
            amount: Some(amount.into()),
            currency: Some(currency.into()),
            unit: Some(BillingUnit::Call),
            tiers: None,
            free_quota: None,
            subscription_id: None,
        }
    }

    /// Create a tiered pricing rule
    pub fn tiered(currency: impl Into<String>, tiers: Vec<PricingTier>) -> Self {
        Self {
            model: PricingModel::Tiered,
            amount: None,
            currency: Some(currency.into()),
            unit: Some(BillingUnit::Call),
            tiers: Some(tiers),
            free_quota: None,
            subscription_id: None,
        }
    }

    /// Add a free quota to this rule
    pub fn with_free_quota(mut self, quota: FreeQuota) -> Self {
        self.free_quota = Some(quota);
        self
    }
}

/// Pricing model type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PricingModel {
    /// No charge
    Free,
    /// Charged per invocation
    PerCall,
    /// Fixed recurring fee
    Subscription,
    /// Price decreases with volume
    Tiered,
    /// Charged based on usage metrics
    Metered,
}

/// Billing unit for pricing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BillingUnit {
    Call,
    Request,
    Token,
    Byte,
    Second,
    Month,
}

impl Default for BillingUnit {
    fn default() -> Self {
        Self::Call
    }
}

/// Pricing tier for tiered pricing model.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PricingTier {
    /// Upper bound for this tier (or "unlimited")
    pub up_to: TierLimit,

    /// Price per unit in this tier
    pub amount: String,
}

impl PricingTier {
    pub fn new(up_to: impl Into<TierLimit>, amount: impl Into<String>) -> Self {
        Self {
            up_to: up_to.into(),
            amount: amount.into(),
        }
    }
}

/// Tier limit - either a number or "unlimited".
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum TierLimit {
    Count(u64),
    Unlimited,
}

impl From<u64> for TierLimit {
    fn from(n: u64) -> Self {
        TierLimit::Count(n)
    }
}

impl From<&str> for TierLimit {
    fn from(s: &str) -> Self {
        if s == "unlimited" {
            TierLimit::Unlimited
        } else {
            TierLimit::Count(s.parse().unwrap_or(0))
        }
    }
}

/// Free quota configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FreeQuota {
    /// Number of free units
    pub amount: u64,

    /// Unit type for the quota
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<BillingUnit>,

    /// Reset period
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<QuotaPeriod>,
}

impl FreeQuota {
    pub fn new(amount: u64, period: QuotaPeriod) -> Self {
        Self {
            amount,
            unit: Some(BillingUnit::Call),
            period: Some(period),
        }
    }
}

/// Quota reset period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuotaPeriod {
    Hour,
    Day,
    Week,
    Month,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_rule_per_call() {
        let rule = PricingRule::per_call("0.003", "USD");
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("\"model\":\"per_call\""));
        assert!(json.contains("\"amount\":\"0.003\""));
    }

    #[test]
    fn test_pricing_rule_tiered() {
        let rule = PricingRule::tiered(
            "USD",
            vec![
                PricingTier::new(1000u64, "0.01"),
                PricingTier::new(10000u64, "0.008"),
                PricingTier::new("unlimited", "0.005"),
            ],
        );
        let json = serde_json::to_string_pretty(&rule).unwrap();
        assert!(json.contains("\"model\": \"tiered\""));
    }

    #[test]
    fn test_pricing_get_tool() {
        let mut pricing = Pricing::with_default(PricingRule::free());
        pricing.add_tool("get_forecast", PricingRule::per_call("0.003", "USD"));

        assert!(pricing.get_tool_pricing("get_forecast").is_some());
        assert_eq!(
            pricing.get_tool_pricing("get_forecast").unwrap().model,
            PricingModel::PerCall
        );

        // Falls back to default
        assert_eq!(
            pricing.get_tool_pricing("unknown_tool").unwrap().model,
            PricingModel::Free
        );
    }
}
