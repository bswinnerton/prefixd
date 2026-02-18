# ADR 013: Dry-Run Mode

## Status

Accepted

## Date

2026-01-15

## Context

Deploying a system that automatically announces BGP FlowSpec rules to production routers is inherently risky. A misconfigured playbook, bad inventory data, or unexpected detector behavior could cause unintended traffic drops.

Operators need a way to:
1. Validate the full pipeline (event → policy → guardrails → FlowSpec rule) without affecting production traffic
2. Onboard new detectors and tune confidence thresholds safely
3. Test playbook changes before going live

## Decision

prefixd supports a global `mode` setting with two values:

- **`dry_run`** -- Full pipeline executes, mitigations are created in the database, metrics are emitted, audit logs are written, but **no BGP announcements are made**. GoBGP is never contacted.
- **`enforced`** -- Full pipeline including BGP announcements to GoBGP.

```yaml
# configs/prefixd.yaml
mode: dry_run   # or: enforced
```

The mode is visible in the health endpoint, dashboard, and logs so operators always know whether rules are being announced.

## Consequences

**Positive:**
- Safe onboarding: run dry-run in production for days/weeks before enabling enforcement
- Playbook testing: validate new policies against real traffic patterns without risk
- Detector tuning: see what mitigations would be created without affecting traffic
- Incident response: switch to dry-run to pause all announcements without shutting down
- Full observability in both modes: metrics, audit log, and dashboard work identically

**Negative:**
- Dry-run doesn't exercise the GoBGP code path (a bug in announcement logic won't surface)
- No per-playbook or per-customer dry-run (it's global only)
- Switching modes requires a config change and reload (no API toggle)

**Future:**
- Per-playbook dry-run override (new playbooks start in dry-run, graduate to enforced)
- API endpoint to toggle mode without config file edit
