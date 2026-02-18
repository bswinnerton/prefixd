# Architecture Decision Records

We use ADRs to document significant architectural and design decisions for prefixd. Each ADR describes the context, decision, and consequences of a choice that affects the system's architecture.

Format follows [Michael Nygard's template](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions).

## Index

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [001](001-gobgp-sidecar.md) | Use GoBGP as a sidecar instead of native BGP | Accepted | 2026-01-15 |
| [002](002-flowspec-only-afi-safi.md) | FlowSpec-only AFI-SAFI for router peers | Accepted | 2026-02-05 |
| [003](003-fail-open-ttl.md) | Fail-open design with mandatory TTLs | Accepted | 2026-01-15 |
| [004](004-destination-prefix-32-only.md) | Restrict FlowSpec to /32 destination prefixes | Accepted | 2026-01-15 |
| [005](005-nginx-single-origin.md) | Nginx reverse proxy as single entrypoint | Accepted | 2026-02-18 |
| [006](006-runtime-url-derivation.md) | Derive frontend URLs at runtime, not build time | Accepted | 2026-02-18 |
| [007](007-trait-based-bgp-abstraction.md) | Trait-based BGP abstraction for testing | Accepted | 2026-01-15 |
| [008](008-session-auth-plus-bearer.md) | Hybrid auth: session cookies + bearer tokens | Accepted | 2026-01-28 |
