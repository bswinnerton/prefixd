# ADR 003: Fail-Open Design with Mandatory TTLs

## Status

Accepted

## Date

2026-01-15

## Context

A DDoS mitigation system that fails closed (leaves rules permanently installed) is more dangerous than one that fails open (rules expire). If prefixd crashes, loses its database, or has a bug in rule management, permanent FlowSpec rules could:

- Block legitimate traffic indefinitely
- Cause customer outages that outlast the original attack
- Require manual router intervention to clear

## Decision

All mitigations **must** have a TTL. There is no way to create a permanent FlowSpec rule through prefixd. The reconciliation loop enforces expiry, and if prefixd itself dies, GoBGP's session to the router will eventually drop (BGP hold timer), clearing all announced routes.

The safety chain:
1. **Application TTL** - prefixd expires mitigations and sends withdrawals
2. **BGP hold timer** - If prefixd/GoBGP dies, the router clears routes after hold time (default 90s)
3. **Operator withdrawal** - `prefixdctl mitigations withdraw <id>` for manual override

## Consequences

**Positive:**
- No mitigation can outlive its usefulness without active renewal
- System crash = all mitigations eventually clear = traffic restored
- Operators have confidence that the system won't cause permanent damage
- Reconciliation loop catches and withdraws any rules that should have expired

**Negative:**
- Long-running mitigations need periodic extension (detectors must re-send events)
- Short TTLs increase event volume; long TTLs delay cleanup
- No "permanent safelist via FlowSpec" use case (use router static config instead)
