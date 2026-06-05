# Payment Manifest

The payment manifest is the heart of `mcp-pay`. It is a JSON document an MCP
server publishes to declare what it charges, which payment rails it accepts,
and what payment-specific SLAs it guarantees.

## Location

The manifest MUST be served at:

```
/.well-known/mcp/pay.json
```

This path lives alongside the MCP Server Card (SEP-2127) namespace:

```
/.well-known/mcp/server-card   ← identity, remotes, capabilities (SEP-2127)
/.well-known/mcp/pay.json      ← pricing, payment rails, SLA (mcp-pay)
```

The server MUST:

- serve the manifest with `Content-Type: application/json`
- include `Access-Control-Allow-Origin: *` for browser clients
- support HTTP GET requests

The server SHOULD include `Cache-Control` headers — the reference server uses
`public, max-age=3600`.

## Example Manifest

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

## Root Object Fields

| Field         | Type            | Required | Description                          |
|---------------|-----------------|----------|--------------------------------------|
| `$schema`     | string          | No       | JSON Schema URL for validation       |
| `mcp_pay`     | string          | Yes      | Schema version (e.g., `"0.1"`)       |
| `server_card` | string          | No       | Path to MCP Server Card              |
| `pricing`     | Pricing         | Yes      | Pricing configuration                |
| `accepts`     | PaymentRail[]   | Yes      | Accepted payment rails               |
| `payment_sla` | PaymentSla      | No       | Payment-specific SLA                 |
| `stats`       | PaymentStats    | No       | Payment statistics                   |
| `extensions`  | object          | No       | Vendor extensions                    |

## Pricing

Pricing is layered. At least one of `default`, `tools`, `resources`, or `prompts`
MUST be present.

```json
{
  "default": { "model": "free" },
  "tools": { "get_forecast": { "model": "per_call", "amount": "0.003", "currency": "USD" } },
  "resources": { "weather/*": { "model": "free" } },
  "prompts": { "summarize": { "model": "per_call", "amount": "0.001", "currency": "USD" } }
}
```

### PricingRule

| Field             | Type            | Required | Notes                                                  |
|-------------------|-----------------|----------|--------------------------------------------------------|
| `model`           | string          | Yes      | `free`, `per_call`, `subscription`, `tiered`, `metered` |
| `amount`          | string          | No       | Decimal price (e.g. `"0.003"`)                         |
| `currency`        | string          | No       | ISO 4217 or token symbol (`"USD"`, `"USDC"`)           |
| `unit`            | string          | No       | `call`, `request`, `token`, `byte`, `second`, `month`  |
| `tiers`           | PricingTier[]   | No       | Required for tiered pricing                            |
| `free_quota`      | FreeQuota       | No       | Free tier allowance                                    |

### Tiered Pricing

The reference server demonstrates tiered pricing for `get_historical`:

```rust
PricingRule::tiered(
    "USD",
    vec![
        PricingTier::new(1000u64, "0.01"),
        PricingTier::new(10000u64, "0.008"),
        PricingTier::new(TierLimit::Unlimited, "0.005"),
    ],
);
```

Each tier has an `up_to` bound (a number or `"unlimited"`) and a per-unit
`amount`.

## Accepted Rails

The `accepts` array lists `PaymentRail` entries. See [Payment Rails](payment-rails.md)
for the full field reference and per-rail conventions.

## Payment SLA

| Field                     | Type   | Description                              |
|---------------------------|--------|------------------------------------------|
| `settlement_time_seconds` | object | Map of rail → seconds                    |
| `refund_policy`           | string | `no_refunds`, `full_refund_on_failure`, `pro_rated`, `time_limited`, `custom` |
| `escrow_available`        | bool   | Whether escrow is supported              |
| `dispute_contact`         | string | Contact for disputes                     |
| `max_payment_usd`         | string | Maximum payment amount                   |
| `min_payment_usd`         | string | Minimum payment amount                   |

## Payment Stats

Optional dynamic stats can be attached to inform discovery ranking:

| Field                | Type    | Description                  |
|----------------------|---------|------------------------------|
| `total_transactions` | integer | Total completed transactions |
| `total_volume_usd`   | string  | Total volume processed       |
| `unique_payers`      | integer | Number of unique payers      |
| `avg_settlement_ms`  | integer | Average settlement time      |
| `success_rate`       | string  | Success rate percentage      |
| `last_updated`       | string  | ISO 8601 timestamp           |

## Building a Manifest in Rust

```rust
use mcp_pay_schema::{
    McpPayManifest, Pricing, PricingRule, PaymentRail, PaymentSla, PaymentStats,
};

let mut pricing = Pricing::with_default(PricingRule::free());
pricing.add_tool("get_forecast", PricingRule::per_call("0.003", "USD"));

let manifest = McpPayManifest::new(
    "0.1",
    pricing,
    vec![PaymentRail::x402_base_usdc("0x1234...")],
)
.with_server_card("/.well-known/mcp/server-card.json")
.with_sla(PaymentSla::crypto_default())
.with_stats(
    PaymentStats::with_transactions(0, "0.00", 0)
        .with_success_rate("100.0")
        .with_avg_settlement(2000),
);
```

The well-known constants are exposed from `mcp_pay_schema`:

- `SCHEMA_VERSION` — `"0.1"`
- `SCHEMA_URL` — `"https://mcp-pay.io/schema/v0.1/pay.schema.json"`
- `WELL_KNOWN_PATH` — `"/.well-known/mcp/pay.json"`
