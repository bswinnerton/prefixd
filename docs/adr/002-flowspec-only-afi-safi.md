# ADR 002: FlowSpec-Only AFI-SAFI for Router Peers

## Status

Accepted

## Date

2026-02-05

## Context

During Juniper cJunosEvolved testing, BGP sessions failed to establish with Open Message Error subcode 7 (Unsupported Capability). Root cause: GoBGP was advertising both `ipv4-unicast` (AFI 1, SAFI 1) and `ipv4-flowspec` (AFI 1, SAFI 133) capabilities to the router peer.

Juniper's FlowSpec implementation rejects peers that advertise unicast alongside flowspec in the same session. This is a known vendor behavior -- the router expects a dedicated FlowSpec peer, not a mixed-capability session.

## Decision

Configure GoBGP neighbors for router peers with **only** FlowSpec address families. No unicast, no multicast, no VPN families.

```toml
[[neighbors]]
  [neighbors.config]
    neighbor-address = "172.30.31.3"
    peer-as = 65020
  [[neighbors.afi-safis]]
    [neighbors.afi-safis.config]
      afi-safi-name = "ipv4-flowspec"
```

## Consequences

**Positive:**
- Juniper, FRR, and likely all vendors accept FlowSpec-only peers without issue
- Cleaner separation of concerns: prefixd's GoBGP only does FlowSpec, router's full table comes from other peers
- Avoids accidentally leaking routes from GoBGP into the router's FIB

**Negative:**
- If we ever need to advertise unicast routes (e.g., RTBH /32 blackholes), we'd need a separate peer group or session
- Must be documented clearly since GoBGP's default is to advertise all families

**Lessons:**
- Always test BGP capability negotiation with real vendor implementations, not just RFC compliance
- Open Message Error subcode 7 is a common symptom of AFI-SAFI mismatch across vendors
