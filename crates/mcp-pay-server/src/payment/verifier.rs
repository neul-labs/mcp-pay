//! Payment verification trait and types.
//!
//! This module defines the abstraction layer for payment verification
//! across different payment rails (x402, MPP, card, etc.).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during payment verification.
#[derive(Debug, Error)]
pub enum PaymentError {
    #[error("Invalid payment signature")]
    InvalidSignature,

    #[error("Payment amount insufficient: required {required}, got {actual}")]
    InsufficientAmount { required: String, actual: String },

    #[error("Payment expired")]
    Expired,

    #[error("Unsupported payment rail: {0}")]
    UnsupportedRail(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid payment payload: {0}")]
    InvalidPayload(String),
}

/// Price requirement for a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRequirement {
    /// Price amount as decimal string
    pub amount: String,

    /// Currency (USD, USDC, etc.)
    pub currency: String,

    /// Resource identifier (tool name, path, etc.)
    pub resource: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl PriceRequirement {
    pub fn new(resource: impl Into<String>, amount: impl Into<String>, currency: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            amount: amount.into(),
            currency: currency.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Payment receipt returned after successful verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentReceipt {
    /// Transaction hash (if on-chain)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,

    /// Settlement timestamp (Unix epoch seconds)
    pub settled_at: u64,

    /// Amount paid
    pub amount: String,

    /// Currency
    pub currency: String,

    /// Payment rail used
    pub rail: String,

    /// Network (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

impl PaymentReceipt {
    /// Encode receipt as base64 for HTTP header.
    pub fn to_header_value(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &json)
    }

    /// Decode receipt from base64 header value.
    pub fn from_header_value(value: &str) -> Result<Self, PaymentError> {
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, value)
            .map_err(|e| PaymentError::InvalidPayload(e.to_string()))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| PaymentError::InvalidPayload(e.to_string()))
    }
}

/// x402 Payment Required response (matches x402 spec).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequired {
    /// x402 protocol version
    pub x402_version: u32,

    /// Accepted payment schemes
    pub accepts: Vec<PaymentRequirement>,
}

/// Individual payment requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequirement {
    /// Payment scheme (e.g., "exact")
    pub scheme: String,

    /// Network identifier (CAIP-2)
    pub network: String,

    /// Maximum amount required
    pub max_amount_required: String,

    /// Resource being paid for
    pub resource: String,

    /// Recipient address
    pub pay_to: String,

    /// Asset/token
    pub asset: String,

    /// Maximum timeout in seconds
    #[serde(default = "default_timeout")]
    pub max_timeout_seconds: u64,

    /// Extra data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

fn default_timeout() -> u64 {
    60
}

impl PaymentRequired {
    /// Encode as base64 for HTTP header.
    pub fn to_base64(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &json)
    }
}

/// Trait for payment verification.
///
/// Implementations handle verification for specific payment rails.
#[async_trait]
pub trait PaymentVerifier: Send + Sync {
    /// Verify a payment payload and return a receipt.
    ///
    /// The `payment_payload` is typically base64-encoded JSON from the
    /// `X-PAYMENT` header.
    async fn verify(
        &self,
        payment_payload: &str,
        requirement: &PriceRequirement,
    ) -> Result<PaymentReceipt, PaymentError>;

    /// Create a payment-required challenge.
    ///
    /// This is returned in the 402 response body and headers.
    fn create_payment_required(&self, requirement: &PriceRequirement) -> PaymentRequired;

    /// Get the payment rail type.
    fn rail(&self) -> &str;

    /// Check if this verifier supports a given rail.
    fn supports(&self, rail: &str) -> bool {
        self.rail() == rail
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_requirement() {
        let req = PriceRequirement::new("get_forecast", "0.003", "USD")
            .with_description("7-day weather forecast");

        assert_eq!(req.resource, "get_forecast");
        assert_eq!(req.amount, "0.003");
        assert_eq!(req.description, Some("7-day weather forecast".to_string()));
    }

    #[test]
    fn test_payment_receipt_encoding() {
        let receipt = PaymentReceipt {
            tx_hash: Some("0xabc123".to_string()),
            settled_at: 1234567890,
            amount: "0.003".to_string(),
            currency: "USDC".to_string(),
            rail: "x402".to_string(),
            network: Some("eip155:8453".to_string()),
        };

        let encoded = receipt.to_header_value();
        let decoded = PaymentReceipt::from_header_value(&encoded).unwrap();

        assert_eq!(decoded.amount, "0.003");
        assert_eq!(decoded.tx_hash, Some("0xabc123".to_string()));
    }
}
