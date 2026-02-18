# ADR 005: Nginx Reverse Proxy as Single Entrypoint

## Status

Accepted

## Date

2026-02-18

## Context

The original docker-compose exposed multiple ports: 8080 (API), 3000 (dashboard), 9090 (metrics). This caused several problems:

1. **WebSocket routing** - The dashboard needed to connect to the backend's WebSocket endpoint, but Next.js cannot proxy WebSocket upgrades through its API routes. We tried `NEXT_PUBLIC_PREFIXD_WS` (build-time env var), then `next.config.mjs` rewrites, but both were fragile.
2. **CORS complexity** - Different origins (port 3000 vs 8080) required CORS configuration for cookies and credentials.
3. **Deployment friction** - Users had to know which port served what, and remote deployments needed build-time URL configuration.

## Decision

Add nginx as a reverse proxy in docker-compose. All user-facing traffic goes through port 80:

- `/v1/*` routes to the prefixd API (including WebSocket upgrade for `/v1/ws/*`)
- `/metrics` routes to the prefixd API
- Everything else routes to the Next.js dashboard

Internal services (prefixd, dashboard) use `expose` instead of `ports` -- they're only accessible within the Docker network.

## Consequences

**Positive:**
- Single origin eliminates CORS issues entirely
- WebSocket works transparently through nginx's `proxy_set_header Upgrade` support
- No build-time URL configuration needed -- `window.location` is always correct
- Matches production deployment pattern (every real deployment uses a reverse proxy)
- Users access everything on one URL: `http://localhost`

**Negative:**
- One more container in docker-compose (minimal resource overhead)
- Port 80 may conflict with existing services on the host (documented in deployment guide)
- Direct API access for debugging requires `docker exec` or adding temporary port mappings

**Alternatives considered:**
- Next.js `rewrites()` for WebSocket proxy -- doesn't work reliably in standalone mode
- Next.js custom server with `http-proxy-middleware` -- adds Node.js complexity, non-standard
- Caddy instead of nginx -- good option but nginx is more universally understood
