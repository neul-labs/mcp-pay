# Payment Flow

The `mcp-pay` payment flow is built on HTTP 402 and is compatible with x402.
The reference server implements it end to end with a facilitator-backed
verifier.

## Discovery

1. The client fetches `/.well-known/mcp/pay.json`.
2. The client parses pricing for the desired tools.
3. The client selects a preferred payment rail from `accepts`.

## Request Diagram

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

## Payment Required (HTTP 402)

When a client calls a paid tool without payment:

1. The server responds with `HTTP 402 Payment Required`.
2. The response includes an `X-PAYMENT-REQUIRED` header (base64 JSON).
3. The response body contains the payment requirements:

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

The reference server uses `x402_version: 2`, `scheme: "exact"`, and a fixed
`max_timeout_seconds` of `60`. If a `CONTRACT` is configured, it is attached
to each requirement under an `extra` object as `{ "contract": "0x…" }`.

## Payment Submission

1. The client constructs a payment proof.
2. The client sends the request again with an `X-PAYMENT` header (base64 JSON).
3. The server verifies the payment with the facilitator.
4. On success the server returns `200 OK` with `X-PAYMENT-RESPONSE`.
5. On failure the server returns `402 Payment Required` with `X-PAYMENT-ERROR`.

### X402PaymentPayload Shape

The reference verifier expects this payload (after base64 decoding):

```json
{
  "x402_version": 2,
  "scheme": "exact",
  "network": "eip155:8453",
  "payload": {
    "signature": "0x…",
    "authorization": "…",
    "amount": "0.003",
    "nonce": "…",
    "expiry": 1717678800
  }
}
```

`signature` is required; `authorization`, `amount`, `nonce`, and `expiry` are
optional.

## Server-Side Verification

`X402Verifier::verify` does the following:

1. Base64-decode and JSON-parse the `X-PAYMENT` payload.
2. Reject the request if the payload network does not match the server's
   configured `NETWORK`.
3. POST the payload to `<X402_FACILITATOR>/verify`.
4. Reject if the facilitator returns a non-success HTTP status or
   `{ "valid": false }`.
5. Return a `PaymentReceipt` containing the facilitator-provided `tx_hash`,
   a `settled_at` epoch second, plus the request's amount, currency, rail
   (`"x402"`), and network.

The receipt is serialised into the `X-PAYMENT-RESPONSE` header on the success
response.

## Error Responses

When verification fails the handler returns:

```http
HTTP/1.1 402 Payment Required
Content-Type: application/json
X-PAYMENT-ERROR: <message>

{
  "error": "payment_invalid",
  "message": "<verifier error>"
}
```

Errors surfaced by the verifier include:

- `InvalidPayload` — base64 or JSON parsing failed.
- `VerificationFailed` — network mismatch or facilitator rejection.
- `NetworkError` — couldn't reach the facilitator.
- `InvalidSignature` — facilitator returned `valid: false`.

## Security Considerations

From the specification:

1. **Payment data is public** — the manifest contains no secrets.
2. **HTTPS required** — payment endpoints MUST use HTTPS.
3. **Rate limiting** — servers SHOULD rate-limit manifest requests.
4. **Payment verification** — always verify payments with the facilitator.
