# ADR 014: Allowlist-Based Config Redaction

## Status

Accepted

## Date

2026-02-18

## Context

The `GET /v1/config/settings` endpoint exposes the running daemon configuration for operational visibility. The initial implementation serialized the entire `Settings` struct to JSON and then removed specific sensitive fields (denylist approach).

This is fragile: when new fields are added to `Settings` (e.g., LDAP bind passwords, RADIUS shared secrets, TLS private key paths), they are exposed by default unless a developer remembers to add a redaction rule.

## Decision

Switch to an allowlist approach. The config settings handler explicitly constructs the response JSON from a curated set of safe fields. New fields added to the `Settings` struct are hidden by default and must be explicitly added to the allowlist to be exposed.

Omitted fields include:
- TLS certificate/key paths (filesystem layout disclosure)
- All `*_env` fields (reveals which environment variables contain secrets)
- LDAP configuration (bind DN, URL, password env)
- RADIUS configuration (server addresses, shared secret env)
- GoBGP gRPC endpoint (internal infrastructure)
- BGP router ID (internal network detail)
- Audit log path (filesystem layout)
- Safelist prefixes (shows protected infrastructure IPs; only count is exposed)

## Consequences

- **Safer by default:** New sensitive fields cannot accidentally leak.
- **Requires maintenance:** When adding new operational fields to `Settings`, a developer must add them to the allowlist if they should be visible in the dashboard.
- **Less complete view:** Operators see a curated subset of config, not the raw file. The full config is still available on the server filesystem.
