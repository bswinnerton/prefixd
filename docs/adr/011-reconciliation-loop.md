# ADR 011: Reconciliation Loop (Desired vs Actual State)

## Status

Accepted

## Date

2026-01-20

## Context

BGP FlowSpec rules can drift from desired state in several ways:

- GoBGP restarts and loses its RIB
- A GoBGP announcement silently fails (network blip, gRPC timeout)
- An operator manually adds/removes routes via `gobgp` CLI
- prefixd restarts and misses TTL expiry windows
- A mitigation expires while prefixd is down

Without reconciliation, drift accumulates: rules that should be active are missing, rules that should be expired are still announced, and the operator has no way to know without manually comparing the database and GoBGP RIB.

## Decision

Run a reconciliation loop every 30 seconds that compares desired state (PostgreSQL) against actual state (GoBGP RIB) and corrects any drift. This follows the Kubernetes controller pattern: declare desired state, observe actual state, converge.

The loop:
1. Load active mitigations from PostgreSQL (desired)
2. Load FlowSpec routes from GoBGP RIB via ListPath (actual)
3. Expire mitigations past their TTL (withdraw + mark expired)
4. Re-announce missing rules (in DB, not in RIB)
5. Withdraw orphan rules (in RIB, not in DB)

## Consequences

**Positive:**
- Self-healing: system converges to correct state after any transient failure
- GoBGP restarts are handled automatically (rules re-announced)
- TTL expiry works even if prefixd was down when the TTL elapsed
- Orphan cleanup prevents stale rules from accumulating
- Operators can trust that the database is the source of truth

**Negative:**
- 30-second convergence window means drift can exist briefly
- ListPath on a large RIB adds GoBGP load (mitigated by pagination at 1000 rules)
- Reconciliation must parse FlowSpec NLRI back into mitigations for comparison (complex, error-prone encoding)
- Could cause announcement storms if GoBGP RIB is completely empty after restart

**Future:**
- Pagination for RIBs larger than 1000 entries
- Configurable reconciliation interval
- Metrics for drift detection (how often does reconciliation find discrepancies?)
