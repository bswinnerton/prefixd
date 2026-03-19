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
