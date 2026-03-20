use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Payment-specific SLA guarantees.
///
/// These are SLA metrics specific to payment processing,
/// complementing rate limits in SEP-1960.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct PaymentSla {
    /// Expected settlement time in seconds per rail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_time_seconds: Option<HashMap<String, u64>>,

    /// Refund policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_policy: Option<RefundPolicy>,

    /// Whether escrow is available for payments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escrow_available: Option<bool>,

    /// Contact for payment disputes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispute_contact: Option<String>,

    /// Maximum payment amount in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_payment_usd: Option<String>,

    /// Minimum payment amount in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_payment_usd: Option<String>,
}

impl PaymentSla {
    /// Create a new payment SLA
    pub fn new() -> Self {
        Self::default()
    }

    /// Set settlement times for rails
    pub fn with_settlement_times(mut self, times: HashMap<String, u64>) -> Self {
        self.settlement_time_seconds = Some(times);
        self
    }

    /// Add a settlement time for a specific rail
    pub fn add_settlement_time(&mut self, rail: impl Into<String>, seconds: u64) {
        self.settlement_time_seconds
            .get_or_insert_with(HashMap::new)
            .insert(rail.into(), seconds);
    }

    /// Set refund policy
    pub fn with_refund_policy(mut self, policy: RefundPolicy) -> Self {
        self.refund_policy = Some(policy);
        self
    }

    /// Enable escrow
    pub fn with_escrow(mut self, available: bool) -> Self {
        self.escrow_available = Some(available);
        self
    }

    /// Set dispute contact
    pub fn with_dispute_contact(mut self, contact: impl Into<String>) -> Self {
        self.dispute_contact = Some(contact.into());
        self
    }

    /// Create a default SLA for crypto payments
    pub fn crypto_default() -> Self {
        let mut times = HashMap::new();
        times.insert("x402".into(), 2);
        times.insert("mpp".into(), 1);
        times.insert("lightning".into(), 5);

        Self {
            settlement_time_seconds: Some(times),
            refund_policy: Some(RefundPolicy::FullRefundOnFailure),
            escrow_available: Some(false),
            dispute_contact: None,
            max_payment_usd: None,
            min_payment_usd: Some("0.001".into()),
        }
    }
}

/// Refund policy type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RefundPolicy {
    /// No refunds available
    NoRefunds,
    /// Full refund if service fails to deliver
    FullRefundOnFailure,
    /// Partial refund based on usage
    ProRated,
    /// Refund within a specific window (see dispute_contact)
    TimeLimited,
    /// Custom policy (see dispute_contact for details)
    Custom,
}

impl std::fmt::Display for RefundPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefundPolicy::NoRefunds => write!(f, "no_refunds"),
            RefundPolicy::FullRefundOnFailure => write!(f, "full_refund_on_failure"),
            RefundPolicy::ProRated => write!(f, "pro_rated"),
            RefundPolicy::TimeLimited => write!(f, "time_limited"),
            RefundPolicy::Custom => write!(f, "custom"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_sla_crypto_default() {
        let sla = PaymentSla::crypto_default();

        let times = sla.settlement_time_seconds.as_ref().unwrap();
        assert_eq!(times.get("x402"), Some(&2));
        assert_eq!(times.get("mpp"), Some(&1));
        assert_eq!(times.get("lightning"), Some(&5));
        assert_eq!(sla.refund_policy, Some(RefundPolicy::FullRefundOnFailure));
    }

    #[test]
    fn test_payment_sla_serialization() {
        let mut sla = PaymentSla::new();
        sla.add_settlement_time("x402", 2);
        sla.refund_policy = Some(RefundPolicy::FullRefundOnFailure);

        let json = serde_json::to_string(&sla).unwrap();
        assert!(json.contains("\"x402\":2"));
        assert!(json.contains("\"full_refund_on_failure\""));
    }
}
