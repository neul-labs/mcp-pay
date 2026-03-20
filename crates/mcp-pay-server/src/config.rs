use std::env;
use std::net::SocketAddr;

/// Server configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP server bind address
    pub http_addr: SocketAddr,

    /// Service identifier
    pub service_name: String,

    /// Payment recipient address (for x402)
    pub pay_to_address: String,

    /// x402 facilitator URL
    pub x402_facilitator: String,

    /// Network identifier (CAIP-2)
    pub network: String,

    /// Asset for payment
    pub asset: String,

    /// Token contract address
    pub contract: Option<String>,

    /// Price per paid call in USD
    pub price_per_call: String,

    /// Enable debug mode
    pub debug: bool,
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            http_addr: env::var("HTTP_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
                .parse()?,

            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "mcp-pay-weather".to_string()),

            pay_to_address: env::var("PAY_TO_ADDRESS")
                .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string()),

            x402_facilitator: env::var("X402_FACILITATOR")
                .unwrap_or_else(|_| "https://x402.org/facilitator".to_string()),

            network: env::var("NETWORK").unwrap_or_else(|_| "eip155:8453".to_string()),

            asset: env::var("ASSET").unwrap_or_else(|_| "USDC".to_string()),

            contract: env::var("CONTRACT").ok(),

            price_per_call: env::var("PRICE_PER_CALL")
                .unwrap_or_else(|_| "0.003".to_string()),

            debug: env::var("DEBUG")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
        })
    }

    /// Create a test configuration.
    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            http_addr: "127.0.0.1:3000".parse().unwrap(),
            service_name: "test-weather".to_string(),
            pay_to_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            x402_facilitator: "https://test.x402.org/facilitator".to_string(),
            network: "eip155:84532".to_string(), // Base Sepolia
            asset: "USDC".to_string(),
            contract: Some("0x036CbD53842c5426634e7929541eC2318f3dCF7e".to_string()),
            price_per_call: "0.001".to_string(),
            debug: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env().expect("Failed to load config from environment")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::test_config();
        assert_eq!(config.service_name, "test-weather");
        assert_eq!(config.asset, "USDC");
    }
}
