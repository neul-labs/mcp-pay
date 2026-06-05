# Getting Started

This page walks through building and running the `mcp-pay` reference server, then
exercising its endpoints.

## Prerequisites

- A working Rust toolchain (the workspace targets edition 2024).
- `curl` and `jq` for poking at the endpoints.

## Clone and Build

```bash
git clone https://github.com/neul-labs/mcp-pay
cd mcp-pay
cargo build --release
```

The workspace contains three crates:

| Crate              | Purpose                                  |
|--------------------|------------------------------------------|
| `mcp-pay-schema`   | Schema types + validation for the manifest |
| `mcp-pay-server`   | Reference HTTP server with payment gating |
| `mcp-pay-cli`      | Registry query CLI (Phase 2 placeholder)  |

## Configure

The server reads configuration from environment variables. All have defaults,
so you can start with none set.

| Variable          | Default                            | Description                  |
|-------------------|------------------------------------|------------------------------|
| `HTTP_ADDR`       | `127.0.0.1:3000`                   | Server bind address          |
| `PAY_TO_ADDRESS`  | `0x0000...0000`                    | Payment recipient            |
| `X402_FACILITATOR`| `https://x402.org/facilitator`     | x402 facilitator URL         |
| `NETWORK`         | `eip155:8453`                      | Chain ID (Base)              |
| `ASSET`           | `USDC`                             | Payment asset                |
| `PRICE_PER_CALL`  | `0.003`                            | Price in USD                 |

Example:

```bash
export PAY_TO_ADDRESS=0x1234567890abcdef1234567890abcdef12345678
export HTTP_ADDR=127.0.0.1:3000
```

The server also accepts `SERVICE_NAME`, `CONTRACT`, and `DEBUG` (see
`crates/mcp-pay-server/src/config.rs`).

## Run

```bash
cargo run -p mcp-pay-server
```

On startup the server logs the address it is listening on and the URL of the
payment manifest.

## Try the Endpoints

```bash
# Get the payment manifest
curl http://localhost:3000/.well-known/mcp/pay.json | jq

# Get current weather (FREE)
curl "http://localhost:3000/api/weather?city=London" | jq

# Get forecast (PAID — returns 402)
curl -i "http://localhost:3000/api/forecast?city=Tokyo"
# HTTP/1.1 402 Payment Required
# X-PAYMENT-REQUIRED: eyJ4NDAyX3ZlcnNpb24i...
```

See [MCP Server Integration](mcp-server.md) for the full endpoint surface and
[Payment Flow](payment-flow.md) for what happens after the 402.

## Using the Schema as a Library

`mcp-pay-schema` can be used independently to build manifests:

```rust
use mcp_pay_schema::{
    McpPayManifest, Pricing, PricingRule, PaymentRail,
};

// Create a manifest
let mut pricing = Pricing::with_default(PricingRule::free());
pricing.add_tool("get_forecast", PricingRule::per_call("0.003", "USD"));

let manifest = McpPayManifest::new(
    "0.1",
    pricing,
    vec![PaymentRail::x402_base_usdc("0x1234...")],
);

// Serialize to JSON
let json = serde_json::to_string_pretty(&manifest)?;
```
