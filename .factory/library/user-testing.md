# User Testing

Testing surface, tools, and resource cost classification for validation.

---

## Validation Surface

### API (curl)
- All backend endpoints testable via curl against Docker stack on port 80 (nginx)
- Auth mode is `none` in dev — no authentication barriers
- Key endpoints to test:
  - POST /v1/events (existing + correlation)
  - POST /v1/signals/alertmanager (new - milestone signal-adapters, not yet implemented)
  - POST /v1/signals/fastnetmon (new - milestone signal-adapters, not yet implemented)
  - GET /v1/signal-groups (new)
  - GET /v1/signal-groups/{id} (new)
  - GET /v1/mitigations/{id} (existing, enhanced with correlation)
  - GET /v1/config/correlation (new - milestone signal-adapters, not yet implemented)
  - PUT /v1/config/correlation (new - milestone signal-adapters, not yet implemented)
  - GET /metrics (Prometheus metrics)

### Browser (agent-browser)
- Dashboard at http://localhost via nginx reverse proxy
- All pages under (dashboard) route group with auth guard
- New Correlation page at /correlation with sub-tabs (milestone correlation-dashboard, not yet implemented)
- Mitigation detail page at /mitigations/[id] with new Correlation section (milestone correlation-dashboard, not yet implemented)
- Dark mode toggle via next-themes

### Docker Stack
- `docker compose up -d` starts full stack
- `docker compose build --no-cache` after code changes
- Health check: `curl http://localhost/v1/health`
- Containers: nginx, prefixd, dashboard, postgres, gobgp, prometheus, grafana

## Validation Concurrency

### curl/API
- All assertions in correlation-engine milestone are API-testable via curl
- Each subagent MUST use a different victim IP range to avoid signal group conflicts
- Assigned IP ranges per subagent (use 203.0.113.x range - TEST-NET-3):
  - Group 1 (lifecycle): 203.0.113.10-19
  - Group 2 (confidence): 203.0.113.20-29
  - Group 3 (corroboration): 203.0.113.30-39
  - Group 4 (api-integration): 203.0.113.40-49
  - Group 5 (metrics): 203.0.113.50-59
  - Group 6 (cross-area): 203.0.113.60-69
  - Group 7 (docs): N/A (file checks only)
- **Max concurrent: 5** (limited by shared database and API server)
- Machine: 128GB RAM, 64 cores, ~20GB baseline usage — plenty of headroom

### agent-browser
- Not needed for correlation-engine milestone (no dashboard assertions)
- **Max concurrent: 5** (for future milestones)

## Setup Notes
- Docker stack must be rebuilt after backend code changes (`docker compose build prefixd`)
- Frontend changes require dashboard rebuild (`docker compose build dashboard`)
- Database migrations run automatically on prefixd startup
- Signal groups require correlation to be enabled in prefixd.yaml config
- Correlation is now enabled in configs/prefixd.yaml with min_sources=1 (backward compat)
- Event format requires: source, victim_ip, vector, timestamp (ISO 8601), plus optional: confidence, bps, pps, ttl_seconds, event_id, top_dst_ports, action
- Config reload: POST /v1/config/reload to hot-reload changes to configs/prefixd.yaml

## Flow Validator Guidance: API

### Event Submission Format
```json
{
  "source": "detector_name",
  "victim_ip": "203.0.113.X",
  "vector": "udp_flood",
  "timestamp": "2026-03-19T18:50:00Z",
  "confidence": 0.8,
  "bps": 1000000,
  "pps": 50000
}
```

### Isolation Rules
- Each subagent uses ONLY IPs from its assigned range (see Concurrency section above)
- Do NOT modify configs/prefixd.yaml — changes affect all subagents
- To test config reload (VAL-ENGINE-021), the assigned subagent should:
  1. Temporarily modify the config
  2. Reload
  3. Test
  4. Restore original config
  5. Reload again
- Withdraw any mitigations you create after testing to avoid quota conflicts

### Shared State Warnings
- Signal groups are keyed by (victim_ip, vector) — different IPs = different groups
- Mitigations table has quotas (max_active_per_customer, per_pop, global)
- The reconciliation loop runs every 30s — expired mitigations will be cleaned up
- Metrics are global counters/histograms — tests should capture before/after deltas

### Known Quirks
- Safelist blocks 10.0.0.0/8 and 192.168.0.0/16 — use 203.0.113.x range for testing
- Default TTL is 120s — set appropriate TTL in events
- Event requires timestamp field (ISO 8601 UTC)
- With min_sources=1, a single event both creates the signal group AND triggers mitigation
- The `ttl_seconds` field is NOT in AttackEventInput — the playbook determines TTL
- Use `action: "ban"` (default) for mitigation, `action: "unban"` for withdraw

### Checking Results
- Signal groups: `curl http://localhost/v1/signal-groups`
- Signal group detail: `curl http://localhost/v1/signal-groups/{id}`
- Mitigations: `curl http://localhost/v1/mitigations`
- Mitigation detail: `curl http://localhost/v1/mitigations/{id}`
- Metrics: `curl http://localhost/metrics`
- OpenAPI: `curl http://localhost/openapi.json`
- Config: `curl http://localhost/v1/config/settings`
- Correlation config: `curl http://localhost/v1/config/correlation`

## Flow Validator Guidance: Signal Adapters (API)

### Alertmanager Webhook Format
The Alertmanager adapter accepts v4 webhook payloads at POST /v1/signals/alertmanager:
```json
{
  "version": "4",
  "status": "firing",
  "alerts": [
    {
      "status": "firing",
      "labels": {
        "vector": "udp_flood",
        "victim_ip": "203.0.113.X",
        "severity": "critical"
      },
      "annotations": {
        "bps": "1000000",
        "pps": "50000"
      },
      "startsAt": "2026-03-19T20:00:00Z",
      "fingerprint": "unique-fingerprint-123"
    }
  ],
  "groupLabels": {},
  "commonLabels": {},
  "commonAnnotations": {},
  "externalURL": "http://alertmanager:9093"
}
```

Key behaviors:
- Returns 200 with per-alert results (not 202)
- Each alert in batch creates separate event with source="alertmanager"
- labels.vector → vector (fallback: labels.alertname)
- labels.victim_ip → victim_ip (fallback: labels.instance with port stripping)
- labels.severity → confidence: critical=0.9, warning=0.7, info=0.5, missing=0.5
- annotations.bps/pps → parsed as optional i64
- fingerprint → external_event_id for dedup
- status="resolved" → action="unban" (withdraw flow)
- Empty/malformed payloads → 400 (not 500)

### FastNetMon Webhook Format
The FastNetMon adapter accepts payloads at POST /v1/signals/fastnetmon:
```json
{
  "action": "ban",
  "ip": "203.0.113.X",
  "alert_scope": "host",
  "attack_details": {
    "attack_uuid": "unique-uuid-123",
    "attack_severity": "high",
    "incoming_udp_pps": 500000,
    "incoming_udp_traffic_bits": 5000000000,
    "total_incoming_pps": 500000,
    "total_incoming_traffic_bits": 5000000000
  }
}
```

Key behaviors:
- Returns 202 with EventResponse (event_id, status, mitigation_id)
- source="fastnetmon" always
- action→confidence: ban=0.9, partial_block=0.7, alert=0.5 (configurable via correlation config)
- Vector classified from traffic breakdown (UDP dominant→udp_flood, SYN dominant→syn_flood, etc.)
- attack_details.attack_uuid → external_event_id for dedup
- Missing/empty action or ip → 400

### Correlation Config API
- GET /v1/config/correlation → returns current config with loaded_at timestamp
- PUT /v1/config/correlation → updates config (admin only, but auth_mode=none bypasses)
- POST /v1/config/reload → reloads all config including correlation

### Auth Notes (auth_mode=none)
- auth_mode is currently "none" — all requests pass authentication
- VAL-ADAPT-010 (auth required) and VAL-ADAPT-015 (admin required) CANNOT be fully tested
  because auth_mode=none means no 401/403 enforcement
- The auth code paths exist and are tested in integration tests but not exercisable via live API

### Signal Adapter IP Ranges
- Alertmanager adapter testing: 203.0.113.110-119
- FastNetMon adapter testing: 203.0.113.120-129
- Config API testing: 203.0.113.130-139
- Cross-area flow testing: 203.0.113.140-149
