# Payment Rails

A `PaymentRail` describes one payment method the server accepts. Rails are
ranked by `priority` (lower is preferred). A manifest's `accepts` field lists
all available rails.

## Supported Rails

| Rail        | Network        | Status       |
|-------------|----------------|--------------|
| `x402`      | Base, Solana   | Implemented  |
| `mpp`       | Tempo          | Planned      |
| `lightning` | Bitcoin        | Planned      |
| `card`      | Stripe         | Planned      |

The `RailType` enum also includes `ach` and `custom` for ACH bank transfers
and vendor-specific rails.

## PaymentRail Fields

| Field          | Type    | Required | Notes                                             |
|----------------|---------|----------|---------------------------------------------------|
| `rail`         | string  | Yes      | `x402`, `mpp`, `lightning`, `card`, `ach`, `custom` |
| `network`      | string  | No       | CAIP-2 chain ID or network name                   |
| `asset`        | string  | No       | Token/asset (e.g. `"USDC"`)                       |
| `contract`     | string  | No       | Token contract address                            |
| `pay_to`       | string  | No       | Recipient address                                 |
| `facilitator`  | string  | No       | Facilitator URL (x402 / MPP)                      |
| `provider`     | string  | No       | Payment provider for card rails                   |
| `lnurl`        | string  | No       | LNURL for Lightning                               |
| `checkout_url` | string  | No       | Checkout URL for card                             |
| `priority`     | integer | No       | Rail preference (lower preferred)                 |

## Convenience Constructors

`mcp-pay-schema` provides ready-made constructors for the common cases.

### x402 on Base with USDC

```rust
PaymentRail::x402_base_usdc("0x1234...");
```

Produces:

| Field       | Value                                                   |
|-------------|---------------------------------------------------------|
| `rail`      | `x402`                                                  |
| `network`   | `eip155:8453`                                           |
| `asset`     | `USDC`                                                  |
| `contract`  | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`            |
| `facilitator` | `https://x402.org/facilitator`                        |
| `priority`  | `0`                                                     |

### MPP on Tempo

```rust
PaymentRail::mpp_tempo("0x1234...");
```

Defaults to `network: "tempo"`, `asset: "USDC"`, `priority: 1`.

### Lightning

```rust
PaymentRail::lightning("lnurl1dp68gurn8ghj7...");
```

Defaults to `network: "bitcoin"`, `priority: 2`.

### Card via Stripe

```rust
PaymentRail::stripe("https://checkout.example.com");
```

Defaults to `provider: "stripe"`, `priority: 3`.

## Builder Methods

```rust
let rail = PaymentRail::x402_base_usdc("0x1234...")
    .with_priority(0)
    .with_facilitator("https://x402.org/facilitator");
```

## Choosing a Rail

The specification calls out four reasons agents need rail awareness:

1. **Discover** which MCP servers require payment.
2. **Compare** pricing across equivalent services.
3. **Choose** appropriate payment rails based on cost, speed, and budget.
4. **Verify** payment-specific SLAs (settlement time, refund policy).

`payment_sla.settlement_time_seconds` is the natural place to publish
expected settlement latency per rail, e.g. `{ "x402": 2, "lightning": 5 }`.
