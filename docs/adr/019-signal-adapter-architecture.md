# ADR 019: Signal Adapter Architecture

## Status

Accepted

## Date

2026-03-19

## Context

prefixd needs to ingest signals from multiple detection and telemetry systems beyond its existing `POST /v1/events` endpoint. The first external integrations are Alertmanager (Prometheus alerting) and FastNetMon (dedicated DDoS detector). Each system has its own payload format, label conventions, and lifecycle semantics (e.g., Alertmanager sends resolved alerts; FastNetMon uses ban/unban actions).

Key questions:

1. **Push vs. pull** — Should prefixd poll external systems for alerts, or should external systems push webhooks to prefixd?
2. **Dedicated endpoints vs. generic** — Should we reuse `POST /v1/events` with adapter-specific fields, or create dedicated endpoints per signal source?
3. **Label mapping** — How should source-specific labels (e.g., Alertmanager's `labels.severity`, `labels.instance`) map to prefixd's internal `AttackEventInput` fields?
4. **Batching** — Alertmanager sends batched alerts in a single webhook call. How should partial failures be handled?
5. **Extensibility** — How easy should it be to add a new signal adapter?

## Decision

### Webhook receivers (push-in model)

We use **push-in webhooks** — external systems push alerts to dedicated prefixd endpoints. This avoids coupling prefixd to external system APIs, avoids polling overhead, and matches how Alertmanager and FastNetMon natively deliver notifications.

### Dedicated endpoints per signal source

Each signal source gets its own endpoint under `/v1/signals/{source}`:

- `POST /v1/signals/alertmanager` — Alertmanager v4 webhook format
- `POST /v1/signals/fastnetmon` — FastNetMon native notify format

We chose dedicated endpoints over reusing `/v1/events` because:

- **Type safety** — Each adapter validates the source-specific payload schema at the HTTP boundary, returning 400 for malformed input (critical for Alertmanager, which won't retry 4xx errors).
- **Clear contracts** — Each endpoint documents exactly what fields are expected from that source, with source-specific defaults (e.g., Alertmanager severity → confidence mapping).
- **Independent evolution** — Adapters can evolve their payload acceptance independently without affecting the core events API.
- **Dedup semantics** — Each source has its own dedup key (Alertmanager uses `fingerprint`, FastNetMon uses its own).

### Internal reuse of event ingestion pipeline

Despite having separate HTTP endpoints, all adapters convert their source-specific payload into `AttackEventInput` and delegate to the existing `handle_ban()` / `handle_unban()` internal functions. This ensures:

- Correlation engine integration (signal groups, source weighting)
- Guardrail checks (safelist, TTL, quotas)
- Policy evaluation (playbook matching)
- BGP announcement/withdrawal
- Audit trail and WebSocket broadcast

### Label mapping pattern

Each adapter defines a deterministic mapping from source-specific labels to `AttackEventInput` fields:

| AttackEventInput field | Alertmanager source | Fallback |
|---|---|---|
| `vector` | `labels.vector` | `labels.alertname` |
| `victim_ip` | `labels.victim_ip` | `labels.instance` (port stripped) |
| `bps` | `annotations.bps` (parsed as i64) | None |
| `pps` | `annotations.pps` (parsed as i64) | None |
| `confidence` | `labels.severity` mapped (critical=0.9, warning=0.7, info=0.5) | 0.5 |
| `action` | `alerts[].status` ("resolved" → "unban", else "ban") | "ban" |
| `event_id` | `alerts[].fingerprint` | None |
| `source` | hardcoded `"alertmanager"` | — |

### Per-alert error handling

Alertmanager sends batched alerts. Each alert is processed independently — a failure in one alert does not abort the batch. The response includes per-alert results with status and optional error messages. The overall HTTP status is always 200 (for well-formed payloads) to prevent Alertmanager from retrying the entire batch.

## Consequences

### Positive

- **Simple integration** — Configure Alertmanager's `webhook_configs` receiver to point at `/v1/signals/alertmanager` and alerts flow into the correlation engine.
- **Type-safe parsing** — Source-specific payloads are validated at ingestion, giving clear error messages for misconfiguration.
- **Extensible** — Adding a new signal adapter is a self-contained task: define the payload struct, write the mapping function, add the handler and route.
- **Correlation-ready** — All adapters feed into the same signal group mechanism, enabling cross-source corroboration (e.g., Alertmanager + FastNetMon signals for the same victim_ip strengthen confidence).

### Negative

- **Endpoint proliferation** — Each new signal source requires a new endpoint. Mitigated by the consistent `/v1/signals/{source}` pattern and reuse of internal pipeline.
- **Mapping maintenance** — Label mappings need documentation and testing for each source. Mitigated by integration tests covering all mapping variants.

### Neutral

- **Authentication** — Signal adapter endpoints require the same authentication as other API endpoints (bearer token or session). Operators must configure their external systems with appropriate credentials.
- **Source identification** — Each adapter sets a hardcoded `source` name (e.g., "alertmanager"), which feeds into the correlation engine's per-source weight configuration.
