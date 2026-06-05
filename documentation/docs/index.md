# mcp-pay

**Payment awareness layer for MCP (Model Context Protocol)**

`mcp-pay` extends the MCP ecosystem with payment capabilities. It provides:

- **Schema** — a JSON manifest (`mcp-pay.json`) for declaring pricing and payment rails
- **Reference Server** — a Rust implementation demonstrating payment-gated MCP tools
- **Protocol** — an HTTP 402 payment flow compatible with x402, MPP, Lightning, and card payments

## The Gap mcp-pay Fills

| Ecosystem    | Discovery               | Payment     |
|--------------|-------------------------|-------------|
| MCP Registry | Tools, resources, prompts | None      |
| x402 Bazaar  | After payment           | x402 only   |
| Tempo MPP    | In-network              | MPP only    |
| **mcp-pay**  | At `.well-known`        | All rails   |

`mcp-pay` bridges these ecosystems by providing a rail-agnostic payment manifest
that works with all payment protocols.

## Key Design Principles

- **Complementary** — works alongside MCP Server Card (SEP-2127)
- **Rail-agnostic** — supports multiple payment protocols
- **Minimal** — only payment-specific fields, no duplication
- **Cacheable** — static manifest, dynamic stats

## Where Next

- [Getting Started](getting-started.md) — clone, build, run the reference server.
- [Payment Manifest](payment-manifest.md) — the `mcp-pay.json` schema, served at
  `/.well-known/mcp/pay.json`.
- [MCP Server Integration](mcp-server.md) — endpoints exposed by the reference
  server and how payment gating is wired up.
- [Payment Flow](payment-flow.md) — the HTTP 402 dance between agent, server,
  and facilitator.
- [Payment Rails](payment-rails.md) — supported rails and their status.
- [Specification](specification.md) — the full draft specification.

## Project Structure

```
mcp-pay/
├── crates/
│   ├── mcp-pay-schema/     # Schema types + validation
│   ├── mcp-pay-server/     # Reference implementation
│   └── mcp-pay-cli/        # Registry queries (Phase 2)
├── specs/
│   ├── schema/
│   │   └── pay.schema.json
│   └── mcp-pay-specification.md
└── README.md
```

## License

Licensed under either of:

- MIT license
- Apache License, Version 2.0

at your option.
