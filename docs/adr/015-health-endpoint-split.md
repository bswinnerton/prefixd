# ADR 015: Split Health Endpoint (Public Liveness + Authenticated Detail)

## Status

Accepted

## Date

2026-02-18

## Context

The `GET /v1/health` endpoint was unauthenticated (used by load balancers, monitoring, and the frontend pre-login) but returned full operational details: BGP peer IP addresses, database connectivity, GoBGP status, active mitigation counts, and uptime.

This exposed internal infrastructure topology and operational state to unauthenticated callers, increasing reconnaissance surface. The endpoint also performed database queries and GoBGP gRPC calls on every request, making it a potential DoS vector.

The frontend needs `auth_mode` from the health endpoint before login to determine whether to show the login form or grant permissions for `auth: none` mode.

## Decision

Split into two endpoints:

- **`GET /v1/health`** (public) -- Lightweight liveness check returning only `{status, version, auth_mode}`. No database or GoBGP calls. Safe for load balancers, monitoring probes, and the frontend auth flow.

- **`GET /v1/health/detail`** (authenticated) -- Full operational health including BGP sessions, database status, GoBGP connectivity, uptime, and active mitigations. Requires authentication.

The CLI (`prefixdctl status`, `prefixdctl peers`) was updated to use `/v1/health/detail`.

## Consequences

- **Reduced attack surface:** Unauthenticated callers learn only that the service is running and what auth mode it uses.
- **Load balancer compatibility:** Public health remains lightweight for health checks at scale.
- **Breaking change for monitoring:** External monitoring that parsed BGP session data from `/v1/health` must be updated to use `/v1/health/detail` with authentication.
- **CLI requires auth:** `prefixdctl` must provide credentials when auth is enabled (was already the case for other commands).

## Migration Checklist

If upgrading from v0.8.2 or earlier:

1. **Monitoring probes** -- Update any Prometheus/Nagios/Datadog checks that parse `bgp_sessions`, `database`, or `gobgp` from `/v1/health`. Switch to `/v1/health/detail` with a bearer token or session cookie.
2. **Load balancers** -- No change needed. `/v1/health` still returns 200 with `{"status":"ok"}` for liveness.
3. **prefixdctl** -- Update to the latest binary. `prefixdctl status` and `prefixdctl peers` now call `/v1/health/detail` automatically.
4. **Custom scripts** -- Any `curl /v1/health | jq .bgp_sessions` one-liners need to change to `curl -H "Authorization: Bearer $TOKEN" /v1/health/detail | jq .bgp_sessions`.
5. **Validation** -- After upgrade, confirm `curl /v1/health` returns the slim `{status, version, auth_mode}` payload and `curl /v1/health/detail` (with auth) returns the full operational response.
