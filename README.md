# mcp-pay

**Payment awareness layer for MCP (Model Context Protocol)**

[![Crates.io](https://img.shields.io/crates/v/mcp-pay-schema)](https://crates.io/crates/mcp-pay-schema)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**Website:** [mcp-pay.neullabs.com](https://mcp-pay.neullabs.com) · **Docs:** [docs.neullabs.com/mcp-pay](https://docs.neullabs.com/mcp-pay) · **Source:** [github.com/neul-labs/mcp-pay](https://github.com/neul-labs/mcp-pay)

mcp-pay extends the MCP ecosystem with payment capabilities. It provides:

- **Schema**: A JSON manifest (`mcp-pay.json`) for declaring pricing and payment rails
- **Reference Server**: A Rust implementation demonstrating payment-gated MCP tools
- **Protocol**: HTTP 402 payment flow compatible with x402, MPP, Lightning, and card payments

## The Gap

| Ecosystem | Discovery | Payment |
|-----------|-----------|---------|
| MCP Registry | Tools, resources, prompts | None |
| x402 Bazaar | After payment | x402 only |
| Tempo MPP | In-network | MPP only |
| **mcp-pay** | At `.well-known` | All rails |

mcp-pay bridges these ecosystems by providing a rail-agnostic payment manifest that works with all payment protocols.

## Quick Start

### Run the Reference Server

```bash
# Clone and build
git clone https://github.com/neul-labs/mcp-pay
cd mcp-pay
cargo build --release

# Configure (optional)
export PAY_TO_ADDRESS=0x1234567890abcdef1234567890abcdef12345678
export HTTP_ADDR=127.0.0.1:3000

# Run
cargo run -p mcp-pay-server
```

### Test the Endpoints

```bash
# Get payment manifest
curl http://localhost:3000/.well-known/mcp/pay.json | jq

# Get weather (FREE)
curl "http://localhost:3000/api/weather?city=London" | jq

# Get forecast (PAID - returns 402)
curl -i "http://localhost:3000/api/forecast?city=Tokyo"
# HTTP/1.1 402 Payment Required
# X-PAYMENT-REQUIRED: eyJ4NDAyX3ZlcnNpb24i...
```

## Project Structure

```
mcp-pay/
├── crates/
│   ├── mcp-pay-schema/     # Schema types + validation
│   ├── mcp-pay-server/     # Reference implementation
│   └── mcp-pay-cli/        # Registry queries (Phase 2)
├── specs/
│   ├── schema/
│   │   └── mcp-pay.schema.json
│   └── mcp-pay-specification.md
└── README.md
```

## Schema Example

Serve at `/.well-known/mcp/pay.json`:

```json
{
  "mcp_pay": "0.1",
  "server_card": "/.well-known/mcp/server-card.json",
  "pricing": {
    "default": { "model": "free" },
    "tools": {
      "get_forecast": {
        "model": "per_call",
        "amount": "0.003",
        "currency": "USD"
      }
    }
  },
  "accepts": [
    {
      "rail": "x402",
      "network": "eip155:8453",
      "asset": "USDC",
      "pay_to": "0x1234..."
    },
    {
      "rail": "lightning",
      "lnurl": "lnurl1dp68gurn8ghj7..."
    }
  ],
  "payment_sla": {
    "settlement_time_seconds": { "x402": 2, "lightning": 5 },
    "refund_policy": "full_refund_on_failure"
  }
}
```

## Usage

### As a Library

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

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HTTP_ADDR` | `127.0.0.1:3000` | Server bind address |
| `PAY_TO_ADDRESS` | `0x0...` | Payment recipient |
| `X402_FACILITATOR` | `https://x402.org/facilitator` | x402 facilitator URL |
| `NETWORK` | `eip155:8453` | Chain ID (Base) |
| `ASSET` | `USDC` | Payment asset |
| `PRICE_PER_CALL` | `0.003` | Price in USD |

## Payment Flow

```
Agent                          MCP Server                    Facilitator
  |                                |                              |
  |  GET /api/forecast             |                              |
  |------------------------------->|                              |
  |                                |                              |
  |  402 Payment Required          |                              |
  |  X-PAYMENT-REQUIRED: {...}     |                              |
  |<-------------------------------|                              |
  |                                |                              |
  |  [Create payment proof]        |                              |
  |                                |                              |
  |  GET /api/forecast             |                              |
  |  X-PAYMENT: {...}              |                              |
  |------------------------------->|                              |
  |                                |  POST /verify                |
  |                                |----------------------------->|
  |                                |                              |
  |                                |  { valid: true }             |
  |                                |<-----------------------------|
  |                                |                              |
  |  200 OK                        |                              |
  |  X-PAYMENT-RESPONSE: {...}     |                              |
  |  { forecast: ... }             |                              |
  |<-------------------------------|                              |
```

## Supported Payment Rails

| Rail | Network | Status |
|------|---------|--------|
| x402 | Base, Solana | Implemented |
| MPP | Tempo | Planned |
| Lightning | Bitcoin | Planned |
| Card | Stripe | Planned |

## Specification

See [specs/mcp-pay-specification.md](specs/mcp-pay-specification.md) for the full specification.

Key design principles:
- **Complementary**: Works alongside MCP Server Card (SEP-2127)
- **Rail-agnostic**: Supports multiple payment protocols
- **Minimal**: Only payment-specific fields, no duplication
- **Cacheable**: Static manifest, dynamic stats

## Roadmap

- [x] **Phase 1**: Schema + Reference Server
- [ ] **Phase 2**: Registry Crawler
- [ ] **Phase 3**: Rail Router (cost optimization)
- [ ] **Phase 4**: Submit as SEP to MCP WG

## License

Licensed under the MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT).

## Contributing

Contributions welcome! Please read the specification before submitting PRs.

## Part of the Neul Labs toolchain

Part of the [Neul Labs](https://www.neullabs.com) agent-infrastructure toolchain:

| Project | Description |
| --- | --- |
| [agentvfs](https://agentvfs.neullabs.com) | Workspace runtime and execution boundary for AI agents. |
| [memorg](https://memorg.neullabs.com) | Give your LLM a memory that actually works. |
| [ormai](https://ormai.neullabs.com) | Give your AI agents database access without the risk — safe text-to-SQL. |
| [closegate](https://closegate.neullabs.com) | The policy chokepoint for finance AI agents. |
| [regulus](https://regulus.neullabs.com) | The EU & UK compliance plane for Google ADK. |
