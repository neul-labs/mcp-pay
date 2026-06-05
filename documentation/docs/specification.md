# Specification

The authoritative `mcp-pay` specification lives in
[`specs/mcp-pay-specification.md`](https://github.com/neul-labs/mcp-pay/blob/main/specs/mcp-pay-specification.md)
in the repository. This page summarises the key points and links to the JSON
Schema.

- **Version:** 0.1 (Draft)
- **Status:** Proto-SEP for the MCP Server Card Working Group
- **Authors:** Neul Labs

## Abstract

MCP-Pay extends the Model Context Protocol (MCP) ecosystem with payment
awareness. It defines a standardised JSON manifest (`pay.json`) that MCP
servers can expose to declare pricing, accepted payment rails, and
payment-specific SLAs. This specification complements SEP-2127 (MCP Server
Cards) by adding payment capabilities without duplicating general server
metadata.

## Namespace Alignment

`mcp-pay` uses the `/.well-known/mcp/` namespace being registered with IANA
per SEP-2127:

```
/.well-known/mcp/server-card   ← identity, remotes, capabilities (SEP-2127)
/.well-known/mcp/pay.json      ← pricing, payment rails, SLA (this spec)
```

## Motivation

The MCP Registry provides a standard for agent tool discovery, but has no
payment awareness. Payment protocols like x402 and MPP each have their own
discovery mechanisms that don't integrate with MCP. Agents need a way to:

1. **Discover** which MCP servers require payment.
2. **Compare** pricing across equivalent services.
3. **Choose** appropriate payment rails based on cost, speed, and budget.
4. **Verify** payment-specific SLAs (settlement time, refund policy).

## Manifest Schema

See [Payment Manifest](payment-manifest.md) for the field-level reference.
The machine-readable JSON Schema lives at
[`specs/schema/pay.schema.json`](https://github.com/neul-labs/mcp-pay/blob/main/specs/schema/pay.schema.json)
and is published as
`https://mcp-pay.io/schema/v0.1/pay.schema.json` (via the `SCHEMA_URL`
constant in `mcp-pay-schema`).

## Relationship to MCP Server Card

`mcp-pay` is designed to complement, not replace, SEP-2127:

| MCP Server Card     | MCP-Pay              |
|---------------------|----------------------|
| Server identity     | Payment pricing      |
| Capabilities        | Payment rails        |
| Authentication      | Payment SLA          |
| Transport           | Payment stats        |

Servers SHOULD link to their Server Card via the `server_card` field.

## Roadmap

- [x] **Phase 1** — Schema + Reference Server
- [ ] **Phase 2** — Registry Crawler
- [ ] **Phase 3** — Rail Router (cost optimization)
- [ ] **Phase 4** — Submit as SEP to the MCP Working Group

## References

- [SEP-2127: MCP Server Cards](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/2127)
- [x402 Protocol Specification](https://github.com/coinbase/x402)
- [CAIP-2: Blockchain ID Specification](https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md)
- [RFC 8615: Well-Known URIs](https://tools.ietf.org/html/rfc8615)

## Changelog

- **0.1** — Initial draft specification.
