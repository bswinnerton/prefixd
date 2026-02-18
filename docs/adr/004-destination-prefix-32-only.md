# ADR 004: Restrict FlowSpec to /32 Destination Prefixes

## Status

Accepted

## Date

2026-01-15

## Context

FlowSpec rules can match on destination prefixes of any length. A rule matching `10.0.0.0/8` would affect all traffic to a /8 block. In a DDoS mitigation context, broad prefix matches are extremely dangerous:

- A /24 rate-limit could affect thousands of unrelated customers
- A /16 discard rule is functionally a network outage
- Automated systems making broad prefix decisions is how major outages happen

## Decision

Guardrails enforce that all FlowSpec destination prefixes must be exactly /32 (single IP). This is validated at ingestion time and cannot be overridden by playbooks or operators.

```rust
// src/guardrails/mod.rs
if prefix.prefix_len() != 32 {
    return Err(GuardrailError::PrefixTooWide { ... });
}
```

## Consequences

**Positive:**
- Blast radius of any single rule is exactly one IP address
- Impossible for a bug or misconfiguration to take down a subnet
- Aligns with how most DDoS attacks target specific IPs (VIPs, game servers, DNS resolvers)
- Makes quota enforcement simple: count of /32s per customer

**Negative:**
- Cannot mitigate attacks targeting an entire prefix (e.g., carpet bombing across a /24)
- Carpet bomb mitigation would need one rule per IP, which may hit router FIB limits
- Some operators may want /28 or /27 rules for network-range protection

**Future:**
- If carpet bomb support is needed, we could add a separate guardrail tier with operator approval and audit logging for prefixes shorter than /32
