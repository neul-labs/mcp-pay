//! x402 payment verification.
//!
//! This module implements the x402 protocol for HTTP 402 payments.
//! It communicates directly with x402 facilitators via HTTP.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{
    PaymentError, PaymentReceipt, PaymentRequired, PaymentRequirement, PaymentVerifier,
    PriceRequirement,
};
use crate::config::Config;

/// x402 payment payload (sent by client in X-PAYMENT header).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X402PaymentPayload {
    /// Protocol version
    pub x402_version: u32,

    /// Payment scheme used
    pub scheme: String,

    /// Network
    pub network: String,

    /// Payment proof/signature
    pub payload: X402PayloadData,
}

/// x402 payload data containing the actual payment proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X402PayloadData {
    /// Signature or authorization
    pub signature: String,

    /// Authorization data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,

    /// Amount being paid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// Nonce to prevent replay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,

    /// Expiry timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,
}

/// x402 facilitator verification response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// x402 payment verifier.
///
/// Verifies payments by calling an x402 facilitator endpoint.
pub struct X402Verifier {
    facilitator_url: String,
    pay_to: String,
    network: String,
    asset: String,
    contract: Option<String>,
    client: reqwest::Client,
}

impl X402Verifier {
    /// Create a new x402 verifier from config.
    pub fn new(config: &Config) -> Self {
        Self {
            facilitator_url: config.x402_facilitator.clone(),
            pay_to: config.pay_to_address.clone(),
            network: config.network.clone(),
            asset: config.asset.clone(),
            contract: config.contract.clone(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a verifier with explicit parameters.
    pub fn with_params(
        facilitator_url: impl Into<String>,
        pay_to: impl Into<String>,
        network: impl Into<String>,
        asset: impl Into<String>,
    ) -> Self {
        Self {
            facilitator_url: facilitator_url.into(),
            pay_to: pay_to.into(),
            network: network.into(),
            asset: asset.into(),
            contract: None,
            client: reqwest::Client::new(),
        }
    }

    /// Set the token contract address.
    pub fn with_contract(mut self, contract: impl Into<String>) -> Self {
        self.contract = Some(contract.into());
        self
    }

    /// Parse a payment payload from base64-encoded string.
    fn parse_payload(&self, payload_b64: &str) -> Result<X402PaymentPayload, PaymentError> {
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, payload_b64)
            .map_err(|e| PaymentError::InvalidPayload(format!("Base64 decode error: {}", e)))?;

        serde_json::from_slice(&bytes)
            .map_err(|e| PaymentError::InvalidPayload(format!("JSON parse error: {}", e)))
    }

    /// Call the facilitator to verify a payment.
    async fn verify_with_facilitator(
        &self,
        payload: &X402PaymentPayload,
    ) -> Result<VerifyResponse, PaymentError> {
        let verify_url = format!("{}/verify", self.facilitator_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&verify_url)
            .json(payload)
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(PaymentError::VerificationFailed(format!(
                "Facilitator returned {}: {}",
                status, body
            )));
        }

        response
            .json()
            .await
            .map_err(|e| PaymentError::VerificationFailed(format!("Invalid response: {}", e)))
    }
}

#[async_trait]
impl PaymentVerifier for X402Verifier {
    async fn verify(
        &self,
        payment_payload: &str,
        requirement: &PriceRequirement,
    ) -> Result<PaymentReceipt, PaymentError> {
        // Parse the payment payload
        let payload = self.parse_payload(payment_payload)?;

        // Verify network matches
        if payload.network != self.network {
            return Err(PaymentError::VerificationFailed(format!(
                "Network mismatch: expected {}, got {}",
                self.network, payload.network
            )));
        }

        // Call facilitator to verify
        let verify_result = self.verify_with_facilitator(&payload).await?;

        if !verify_result.valid {
            return Err(PaymentError::InvalidSignature);
        }

        // Create receipt
        Ok(PaymentReceipt {
            tx_hash: verify_result.tx_hash,
            settled_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            amount: requirement.amount.clone(),
            currency: requirement.currency.clone(),
            rail: "x402".to_string(),
            network: Some(self.network.clone()),
        })
    }

    fn create_payment_required(&self, requirement: &PriceRequirement) -> PaymentRequired {
        PaymentRequired {
            x402_version: 2,
            accepts: vec![PaymentRequirement {
                scheme: "exact".to_string(),
                network: self.network.clone(),
                max_amount_required: requirement.amount.clone(),
                resource: requirement.resource.clone(),
                pay_to: self.pay_to.clone(),
                asset: self.asset.clone(),
                max_timeout_seconds: 60,
                extra: self.contract.as_ref().map(|c| {
                    serde_json::json!({
                        "contract": c
                    })
                }),
            }],
        }
    }

    fn rail(&self) -> &str {
        "x402"
    }
}

/// Mock x402 verifier for testing.
#[cfg(test)]
pub struct MockX402Verifier {
    pub should_succeed: bool,
}

#[cfg(test)]
#[async_trait]
impl PaymentVerifier for MockX402Verifier {
    async fn verify(
        &self,
        _payment_payload: &str,
        requirement: &PriceRequirement,
    ) -> Result<PaymentReceipt, PaymentError> {
        if self.should_succeed {
            Ok(PaymentReceipt {
                tx_hash: Some("0xmock_tx_hash".to_string()),
                settled_at: 1234567890,
                amount: requirement.amount.clone(),
                currency: requirement.currency.clone(),
                rail: "x402".to_string(),
                network: Some("eip155:8453".to_string()),
            })
        } else {
            Err(PaymentError::InvalidSignature)
        }
    }

    fn create_payment_required(&self, requirement: &PriceRequirement) -> PaymentRequired {
        PaymentRequired {
            x402_version: 2,
            accepts: vec![PaymentRequirement {
                scheme: "exact".to_string(),
                network: "eip155:8453".to_string(),
                max_amount_required: requirement.amount.clone(),
                resource: requirement.resource.clone(),
                pay_to: "0xmock_address".to_string(),
                asset: "USDC".to_string(),
                max_timeout_seconds: 60,
                extra: None,
            }],
        }
    }

    fn rail(&self) -> &str {
        "x402"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_required_creation() {
        let verifier = X402Verifier::with_params(
            "https://test.facilitator.com",
            "0x1234",
            "eip155:8453",
            "USDC",
        );

        let req = PriceRequirement::new("get_forecast", "0.003", "USD");
        let payment_required = verifier.create_payment_required(&req);

        assert_eq!(payment_required.x402_version, 2);
        assert_eq!(payment_required.accepts.len(), 1);
        assert_eq!(payment_required.accepts[0].max_amount_required, "0.003");
        assert_eq!(payment_required.accepts[0].network, "eip155:8453");
    }

    #[tokio::test]
    async fn test_mock_verifier() {
        let verifier = MockX402Verifier {
            should_succeed: true,
        };

        let req = PriceRequirement::new("test", "0.001", "USD");
        let result = verifier.verify("mock_payload", &req).await;

        assert!(result.is_ok());
        let receipt = result.unwrap();
        assert_eq!(receipt.rail, "x402");
    }
}
