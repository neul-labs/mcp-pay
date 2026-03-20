# MCP-Pay Specification

**Version:** 0.1 (Draft)
**Status:** Proto-SEP for MCP Server Card Working Group
**Authors:** Neul Labs
**Date:** March 2026

## Abstract

MCP-Pay extends the Model Context Protocol (MCP) ecosystem with payment awareness. It defines a standardized JSON manifest (`pay.json`) that MCP servers can expose to declare pricing, accepted payment rails, and payment-specific SLAs. This specification complements SEP-2127 (MCP Server Cards) by adding payment capabilities without duplicating general server metadata.

### Namespace Alignment

This specification uses the `/.well-known/mcp/` namespace being registered with IANA per SEP-2127. The payment manifest lives alongside the server card:

```
/.well-known/mcp/server-card   ← identity, remotes, capabilities (SEP-2127)
/.well-known/mcp/pay.json      ← pricing, payment rails, SLA (this spec)
```

## Motivation

The MCP Registry provides a standard for agent tool discovery, but has no payment awareness. Meanwhile, payment protocols like x402 and MPP each have their own discovery mechanisms that don't integrate with MCP. Agents need a way to:

1. **Discover** which MCP servers require payment
2. **Compare** pricing across equivalent services
3. **Choose** appropriate payment rails based on cost, speed, and budget
4. **Verify** payment-specific SLAs (settlement time, refund policy)

MCP-Pay bridges this gap by providing a payment-specific manifest that works alongside the MCP Server Card.

## Specification

### 1. Manifest Location

The MCP-Pay manifest MUST be served at:

```
/.well-known/mcp/pay.json
```

The server MUST:
- Serve the manifest with `Content-Type: application/json`
- Include `Access-Control-Allow-Origin: *` for browser clients
- Support HTTP GET requests

The server SHOULD:
- Include `Cache-Control` headers (recommended: `public, max-age=3600`)

### 2. Manifest Schema

#### 2.1 Root Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `$schema` | string | No | JSON Schema URL for validation |
| `mcp_pay` | string | Yes | Schema version (e.g., "0.1") |
| `server_card` | string | No | Path to MCP Server Card |
| `pricing` | Pricing | Yes | Pricing configuration |
| `accepts` | PaymentRail[] | Yes | Accepted payment rails |
| `payment_sla` | PaymentSla | No | Payment-specific SLA |
| `stats` | PaymentStats | No | Payment statistics |
| `extensions` | object | No | Vendor extensions |

#### 2.2 Pricing Object

```json
{
  "default": PricingRule,
  "tools": { "tool_name": PricingRule },
  "resources": { "resource_pattern": PricingRule },
  "prompts": { "prompt_name": PricingRule }
}
```

At least one of `default`, `tools`, `resources`, or `prompts` MUST be present.

#### 2.3 PricingRule Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `model` | string | Yes | One of: `free`, `per_call`, `subscription`, `tiered`, `metered` |
| `amount` | string | No | Price as decimal string (e.g., "0.003") |
| `currency` | string | No | ISO 4217 or token symbol (e.g., "USD", "USDC") |
| `unit` | string | No | One of: `call`, `request`, `token`, `byte`, `second`, `month` |
| `tiers` | PricingTier[] | No | For tiered pricing |
| `free_quota` | FreeQuota | No | Free tier allowance |

#### 2.4 PaymentRail Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `rail` | string | Yes | One of: `x402`, `mpp`, `lightning`, `card`, `ach`, `custom` |
| `network` | string | No | CAIP-2 chain ID or network name |
| `asset` | string | No | Token/asset (e.g., "USDC") |
| `contract` | string | No | Token contract address |
| `pay_to` | string | No | Recipient address |
| `facilitator` | string | No | Facilitator URL (x402/MPP) |
| `provider` | string | No | Payment provider (card rails) |
| `lnurl` | string | No | LNURL (Lightning) |
| `checkout_url` | string | No | Checkout URL (card) |
| `priority` | integer | No | Rail preference (lower = preferred) |

#### 2.5 PaymentSla Object

| Field | Type | Description |
|-------|------|-------------|
| `settlement_time_seconds` | object | Map of rail -> seconds |
| `refund_policy` | string | One of: `no_refunds`, `full_refund_on_failure`, `pro_rated`, `time_limited`, `custom` |
| `escrow_available` | boolean | Whether escrow is supported |
| `dispute_contact` | string | Contact for disputes |
| `max_payment_usd` | string | Maximum payment amount |
| `min_payment_usd` | string | Minimum payment amount |

#### 2.6 PaymentStats Object

| Field | Type | Description |
|-------|------|-------------|
| `total_transactions` | integer | Total completed transactions |
| `total_volume_usd` | string | Total volume processed |
| `unique_payers` | integer | Number of unique payers |
| `avg_settlement_ms` | integer | Average settlement time |
| `success_rate` | string | Success rate percentage |
| `last_updated` | string | ISO 8601 timestamp |

### 3. Payment Flow

#### 3.1 Discovery

1. Client fetches `/.well-known/mcp/pay.json`
2. Client parses pricing for desired tools
3. Client selects preferred payment rail from `accepts`

#### 3.2 Payment Required (HTTP 402)

When a client calls a paid tool without payment:

1. Server responds with `HTTP 402 Payment Required`
2. Response includes `X-PAYMENT-REQUIRED` header (base64 JSON)
3. Response body contains payment requirements

```json
{
  "x402_version": 2,
  "accepts": [{
    "scheme": "exact",
    "network": "eip155:8453",
    "max_amount_required": "0.003",
    "resource": "get_forecast",
    "pay_to": "0x...",
    "asset": "USDC",
    "max_timeout_seconds": 60
  }]
}
```

#### 3.3 Payment Submission

1. Client constructs payment proof
2. Client sends request with `X-PAYMENT` header (base64 JSON)
3. Server verifies payment with facilitator
4. On success: returns response with `X-PAYMENT-RESPONSE` header
5. On failure: returns `402` with `X-PAYMENT-ERROR` header

### 4. Relationship to MCP Server Card

MCP-Pay is designed to complement, not replace, SEP-2127:

| MCP Server Card | MCP-Pay |
|-----------------|---------|
| Server identity | Payment pricing |
| Capabilities | Payment rails |
| Authentication | Payment SLA |
| Transport | Payment stats |

Servers SHOULD link to their Server Card via the `server_card` field.

### 5. Security Considerations

1. **Payment data is public**: The manifest contains no secrets
2. **HTTPS required**: Payment endpoints MUST use HTTPS
3. **Rate limiting**: Servers SHOULD rate-limit manifest requests
4. **Payment verification**: Always verify payments with facilitator

### 6. Example Manifest

```json
{
  "$schema": "https://mcp-pay.io/schema/v0.1/mcp-pay.schema.json",
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
      "pay_to": "0x1234...",
      "facilitator": "https://x402.org/facilitator",
      "priority": 0
    }
  ],
  "payment_sla": {
    "settlement_time_seconds": { "x402": 2 },
    "refund_policy": "full_refund_on_failure"
  },
  "stats": {
    "total_transactions": 15420000,
    "success_rate": "99.8"
  }
}
```

## References

- [SEP-2127: MCP Server Cards](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/2127)
- [x402 Protocol Specification](https://github.com/coinbase/x402)
- [CAIP-2: Blockchain ID Specification](https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md)
- [RFC 8615: Well-Known URIs](https://tools.ietf.org/html/rfc8615)

## Changelog

- **0.1**: Initial draft specification
