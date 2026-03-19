# User Testing

Testing surface, tools, and resource cost classification for validation.

---

## Validation Surface

### API (curl)
- All backend endpoints testable via curl against Docker stack on port 80 (nginx)
- Auth mode is `none` in dev — no authentication barriers
- Key endpoints to test:
  - POST /v1/events (existing + correlation)
  - POST /v1/signals/alertmanager (new)
  - POST /v1/signals/fastnetmon (new)
  - GET /v1/signal-groups (new)
  - GET /v1/signal-groups/{id} (new)
  - GET /v1/mitigations/{id} (existing, enhanced with correlation)
  - GET /v1/config/correlation (new)
  - PUT /v1/config/correlation (new)
  - GET /metrics (Prometheus metrics)

### Browser (agent-browser)
- Dashboard at http://localhost via nginx reverse proxy
- All pages under (dashboard) route group with auth guard
- New Correlation page at /correlation with sub-tabs
- Mitigation detail page at /mitigations/[id] with new Correlation section
- Dark mode toggle via next-themes

### Docker Stack
- `docker compose up -d` starts full stack
- `docker compose build --no-cache` after code changes
- Health check: `curl http://localhost/v1/health`
- Containers: nginx, prefixd, dashboard, postgres, gobgp, prometheus, grafana

## Validation Concurrency

### agent-browser
- Machine: 128GB RAM, 64 cores, ~20GB baseline usage
- Usable headroom: ~75GB * 0.7 = ~52GB
- Per agent-browser instance: ~300MB (app is lightweight)
- Dev server (dashboard): ~200MB
- **Max concurrent: 5** (well within budget)

### curl/API
- Negligible resource usage
- **Max concurrent: 5**

## Setup Notes
- Docker stack must be rebuilt after backend code changes (`docker compose build prefixd`)
- Frontend changes require dashboard rebuild (`docker compose build dashboard`)
- Database migrations run automatically on prefixd startup
- Signal groups require correlation to be enabled in prefixd.yaml config
