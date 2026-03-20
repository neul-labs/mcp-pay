use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Payment statistics for discovery and ranking.
///
/// These stats help agents evaluate service reliability
/// and popularity when choosing between providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct PaymentStats {
    /// Total number of completed transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_transactions: Option<u64>,

    /// Total volume processed in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_volume_usd: Option<String>,

    /// Number of unique paying users/agents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_payers: Option<u64>,

    /// Average settlement time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_settlement_ms: Option<u64>,

    /// Payment success rate as percentage string (e.g., "99.8")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_rate: Option<String>,

    /// When these stats were last updated (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub last_updated: Option<String>,

    /// 30-day rolling transaction count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions_30d: Option<u64>,

    /// 30-day rolling volume in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_30d_usd: Option<String>,
}

impl PaymentStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Create stats with transaction data
    pub fn with_transactions(
        total: u64,
        volume_usd: impl Into<String>,
        unique_payers: u64,
    ) -> Self {
        Self {
            total_transactions: Some(total),
            total_volume_usd: Some(volume_usd.into()),
            unique_payers: Some(unique_payers),
            last_updated: Some(Utc::now().to_rfc3339()),
            ..Default::default()
        }
    }

    /// Set success rate
    pub fn with_success_rate(mut self, rate: impl Into<String>) -> Self {
        self.success_rate = Some(rate.into());
        self
    }

    /// Set average settlement time
    pub fn with_avg_settlement(mut self, ms: u64) -> Self {
        self.avg_settlement_ms = Some(ms);
        self
    }

    /// Update the last_updated timestamp
    pub fn touch(&mut self) {
        self.last_updated = Some(Utc::now().to_rfc3339());
    }

    /// Calculate trust score (0.0-1.0) based on stats
    pub fn trust_score(&self) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Success rate contributes up to 0.4
        if let Some(rate) = &self.success_rate {
            if let Ok(r) = rate.parse::<f64>() {
                score += (r / 100.0).min(1.0) * 0.4;
                factors += 1;
            }
        }

        // Transaction volume contributes up to 0.3
        if let Some(txns) = self.total_transactions {
            // Logarithmic scale: 1000 txns = full score
            score += (txns as f64).log10().min(3.0) / 3.0 * 0.3;
            factors += 1;
        }

        // Unique payers contributes up to 0.2
        if let Some(payers) = self.unique_payers {
            // Logarithmic scale: 100 payers = full score
            score += (payers as f64).log10().min(2.0) / 2.0 * 0.2;
            factors += 1;
        }

        // Settlement speed contributes up to 0.1
        if let Some(ms) = self.avg_settlement_ms {
            // Faster is better: < 100ms = full score
            score += (1.0 - (ms as f64 / 10000.0).min(1.0)) * 0.1;
            factors += 1;
        }

        if factors > 0 {
            score
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_stats_creation() {
        let stats = PaymentStats::with_transactions(15420000, "46260.00", 2341)
            .with_success_rate("99.8")
            .with_avg_settlement(1200);

        assert_eq!(stats.total_transactions, Some(15420000));
        assert_eq!(stats.total_volume_usd, Some("46260.00".into()));
        assert_eq!(stats.success_rate, Some("99.8".into()));
    }

    #[test]
    fn test_trust_score() {
        let stats = PaymentStats::with_transactions(10000, "50000.00", 500)
            .with_success_rate("99.5")
            .with_avg_settlement(50);

        let score = stats.trust_score();
        assert!(score > 0.7, "Score should be high for good stats: {}", score);
    }

    #[test]
    fn test_empty_stats_trust_score() {
        let stats = PaymentStats::new();
        assert_eq!(stats.trust_score(), 0.0);
    }

    #[test]
    fn test_stats_serialization() {
        let stats = PaymentStats::with_transactions(1000, "5000.00", 50);
        let json = serde_json::to_string(&stats).unwrap();

        assert!(json.contains("\"total_transactions\":1000"));
        assert!(json.contains("\"total_volume_usd\":\"5000.00\""));
    }
}
