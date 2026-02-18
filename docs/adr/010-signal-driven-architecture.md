# ADR 010: Signal-Driven Architecture (Detectors Signal, prefixd Decides)

## Status

Accepted

## Date

2026-01-15

## Context

DDoS mitigation systems typically fall into two camps:

1. **Integrated detection + mitigation** -- One system detects attacks and applies filters (e.g., Arbor, CloudFlare Magic Transit)
2. **Separated detection and policy** -- Detectors signal events, a policy layer decides what to do

Building detection into prefixd would mean implementing attack classification algorithms (rate-based, ML-based, protocol anomaly), maintaining per-protocol heuristics, and handling the inevitable false positive tuning.

Detection is a solved problem with specialized tools (FastNetMon, Kentik, Prometheus-based alerters, vendor-specific solutions). Policy enforcement is the gap.

## Decision

prefixd is a **policy daemon**, not a detection engine. Detectors are external systems that send structured events to prefixd's API. prefixd evaluates those events against playbooks, applies guardrails, and announces FlowSpec rules.

```
Detector → POST /v1/events → Policy Engine → Guardrails → GoBGP → Routers
```

Detectors are treated as **untrusted** -- they can suggest mitigations, but guardrails have final say on what gets announced.

## Consequences

**Positive:**
- Use the best detector for each environment (FastNetMon for flow-based, Prometheus for metric-based, custom scripts for protocol-specific)
- Detection tuning is independent of mitigation policy
- Multiple detectors can feed into the same policy engine (foundation for v1.5 multi-signal correlation)
- prefixd stays simple and focused on what it does well: policy + BGP
- Detectors don't need to understand BGP, FlowSpec, or router quirks

**Negative:**
- Requires at least one external detector to be useful (no standalone mode)
- Event API is the integration surface -- every detector needs an adapter
- Latency includes detector-to-prefixd HTTP round-trip (typically <10ms on same network)
- No built-in detection means prefixd can't demo itself without external tooling

**Future:**
- v1.5 multi-signal correlation will combine weak signals from multiple detectors into high-confidence decisions
