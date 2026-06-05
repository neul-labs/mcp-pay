# MCP Server Integration

The `mcp-pay-server` crate is a reference implementation of an MCP server with
payment gating. It shows how to wire the manifest, the HTTP 402 flow, and an
x402 facilitator together.

## Endpoints

| Method | Path                          | Description                                |
|--------|-------------------------------|--------------------------------------------|
| GET    | `/.well-known/mcp/pay.json`   | Payment manifest                           |
| GET    | `/api/weather?city=…`         | Current weather (FREE)                     |
| GET    | `/api/forecast?city=…`        | 7-day forecast (PAID — returns 402)        |
| GET    | `/health`                     | Health check (`ok`)                        |

Routes are registered in `crates/mcp-pay-server/src/http/mod.rs`. The router
also applies a permissive CORS layer (`Any` origin/method/headers) and an HTTP
trace layer.

## Manifest Handler

`http::well_known::mcp_pay_json` builds the manifest dynamically from the
server's configuration each request:

- `get_weather` is published as free.
- `get_forecast` is published as `per_call` at `PRICE_PER_CALL` USD.
- `get_historical` is published with a tiered USD price (`0.01` up to 1000,
  `0.008` up to 10000, `0.005` unlimited).
- An x402 rail is added using `NETWORK`, `ASSET`, `CONTRACT`,
  `PAY_TO_ADDRESS`, and `X402_FACILITATOR` from the config.
- A `server_card` link points at `/.well-known/mcp/server-card`.
- A default crypto SLA and zeroed `PaymentStats` are attached.

The response headers are:

```
Content-Type: application/json
Access-Control-Allow-Origin: *
Cache-Control: public, max-age=3600
```

## Payment-Gated Endpoint

`/api/forecast` is gated by the middleware in
`crates/mcp-pay-server/src/http/middleware.rs`. The shape:

```rust
let requirement = PriceRequirement::new(
    "get_forecast",
    &config.price_per_call,
    "USD",
)
.with_description("7-day weather forecast");
```

Behaviour:

1. If the request has no `X-PAYMENT` header, the handler returns
   `402 Payment Required` with:
    - body: JSON `PaymentRequired` from
      `state.verifier.create_payment_required(&requirement)`
    - header: `X-PAYMENT-REQUIRED: <base64-json>`
2. If the request includes `X-PAYMENT`, the handler calls
   `state.verifier.verify(payment_payload, &requirement).await`:
    - On success — returns `200 OK` with the forecast and
      `X-PAYMENT-RESPONSE: <receipt-header>`.
    - On failure — returns `402 Payment Required` with
      `X-PAYMENT-ERROR: <message>` and a JSON `{ "error": "payment_invalid", "message": "…" }`.

The free `/api/weather` endpoint has no payment check.

## Application State

`http::AppState` carries:

- `config: Config` — loaded from env vars (see [Getting Started](getting-started.md)).
- `verifier: Arc<X402Verifier>` — built from the same config.

The verifier is created once in `create_router` and shared across handlers via
`Router::with_state`.

## Tools

Weather tool implementations live in `crates/mcp-pay-server/src/tools/`. The
middleware delegates to `weather::get_current` for the free endpoint and
`weather::get_forecast` for the paid endpoint.

## Tests

The crate ships unit tests that:

- assert `/health` returns `200 OK`,
- assert `/api/weather` returns `200 OK`,
- assert `/api/forecast` returns `402 Payment Required` with an
  `X-PAYMENT-REQUIRED` header when no payment is supplied,
- exercise the manifest handler.

A `MockX402Verifier` is provided for tests where calling a live facilitator is
not desirable.
