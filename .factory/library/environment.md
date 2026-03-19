# Environment

Environment variables, external dependencies, and setup notes.

**What belongs here:** Required env vars, external API keys/services, dependency quirks, platform-specific notes.
**What does NOT belong here:** Service ports/commands (use `.factory/services.yaml`).

---

## Required Environment
- Rust 2024 edition (1.85+)
- Bun 1.3+ for frontend
- Docker Compose for full-stack testing
- PostgreSQL 15+ (via Docker Compose)

## Database
- Connection string: `postgres://prefixd:prefixd@localhost:5432/prefixd` (default in docker-compose)
- Migrations run automatically on startup
- Current: 6 migrations (001-006), mission adds migration 007

## Auth Modes
- Development: `auth_mode: none` (no auth required)
- Production: `credentials`, `bearer`, or `mtls`
