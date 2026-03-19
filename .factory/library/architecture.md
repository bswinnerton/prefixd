# Architecture

Architectural decisions, patterns, and constraints for the multi-signal correlation feature.

---

## Correlation Engine Integration Point

The correlation engine sits between event storage (step 6) and policy evaluation (step 9) in the `handle_ban()` flow in `src/api/handlers.rs`. The flow becomes:

1. Event arrives via POST /v1/events (or /v1/signals/*)
2. Validate, deduplicate, create AttackEvent, store in events table
3. **NEW: Correlation step** — find or create signal group for (victim_ip, vector)
4. Add event to signal group, recompute derived confidence
5. Check corroboration threshold (min_sources, confidence_threshold)
6. If threshold met → proceed to policy evaluation
7. If not met → return accepted (signal recorded, no mitigation yet)

## Existing Dead Code to Replace

- `src/policy/correlation.rs` — `EventCorrelator` is never used in production. The new `src/correlation/` module replaces this conceptually.
- `correlation_window_seconds` in `TimersConfig` — currently parsed but unused. Wire into the new CorrelationConfig.

## Key Design Decisions

- **ADR 018**: Time-windowed grouping, weighted confidence, optional corroboration
- **ADR 019**: Webhook receivers, dedicated endpoints, configurable label mapping

## Alertmanager Webhook Format

Alertmanager v4 payload:
- `version: "4"` (always)
- `alerts[]` — array of individual alerts (batch)
- Each alert: `status` (firing/resolved), `labels` (key-value), `annotations` (key-value), `startsAt`, `endsAt`, `fingerprint`
- `fingerprint` used as external_event_id for dedup
- Resolved alerts → unban/withdraw flow
- Returns 200 on success, 400 on malformed (Alertmanager won't retry 4xx)

## Source Weight System

- Each signal source has a configurable weight (default 1.0)
- derived_confidence = sum(confidence_i * weight_i) / sum(weight_i)
- Unknown sources get weight 1.0
- Source weights defined in correlation.sources config section

## Concurrent-Safe Insert Pattern (CTE)

For tables requiring exactly-one-row semantics under concurrent access (e.g., signal_groups where only one open group per victim_ip+vector should exist), the codebase uses a CTE pattern:

```sql
WITH existing AS (
  SELECT group_id FROM signal_groups
  WHERE victim_ip = $1 AND vector = $2 AND status = 'open'
  LIMIT 1
),
inserted AS (
  INSERT INTO signal_groups (group_id, victim_ip, vector, ...)
  SELECT $3, $1, $2, ...
  WHERE NOT EXISTS (SELECT 1 FROM existing)
  RETURNING group_id
)
SELECT group_id FROM existing
UNION ALL
SELECT group_id FROM inserted
LIMIT 1
```

This pattern (in `src/db/repository.rs`) returns the existing row if found, or inserts a new one. Note: this handles sequential races but does NOT provide true atomic upsert guarantees without a partial unique index on `(victim_ip, vector) WHERE status = 'open'`. For the current low-concurrency use case, it's sufficient.

Compare with the simpler `INSERT ... ON CONFLICT DO NOTHING` used for `signal_group_events.add_event_to_group()` where the (group_id, event_id) primary key provides natural dedup.

## API Response Context Levels

The mitigation API uses two levels of correlation context:

- **List endpoint** (`GET /v1/mitigations`): Returns lightweight summary with `signal_group_id`, `derived_confidence`, `source_count`, `corroboration_met`, but `contributing_sources: []` and `explanation: ""` (empty) for performance.
- **Detail endpoint** (`GET /v1/mitigations/{id}`): Returns full context including populated `contributing_sources` array and human-readable `explanation` string, computed from signal group events.

This is a deliberate performance optimization — the list endpoint avoids N additional queries to fetch per-group event details. API consumers should use the detail endpoint when they need contributing source information.
