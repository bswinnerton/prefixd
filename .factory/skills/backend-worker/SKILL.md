---
name: backend-worker
description: Implements Rust backend features for prefixd (handlers, modules, tests, migrations, docs)
---

# Backend Worker

NOTE: Startup and cleanup are handled by `worker-base`. This skill defines the WORK PROCEDURE.

## When to Use This Skill

Use for features that involve:
- Rust backend code (handlers, modules, config, domain types)
- Database migrations
- API endpoints (handlers + routes + OpenAPI registration)
- Integration tests
- Backend documentation (api.md, configuration.md, ADRs, CHANGELOG)
- Prometheus metrics

## Work Procedure

1. **Read the feature description thoroughly.** Understand preconditions, expected behavior, verification steps, and which validation contract assertions this feature fulfills.

2. **Read AGENTS.md** for mission boundaries, coding conventions, and module structure guidance.

3. **Read existing code** in the area you're modifying. Understand patterns before writing new code. Key files:
   - `src/api/handlers.rs` — all HTTP handlers
   - `src/api/routes.rs` — route registration (shared `api_routes()`)
   - `src/api/openapi.rs` — OpenAPI spec registration
   - `src/config/settings.rs` — Settings struct and config parsing
   - `src/state.rs` — AppState with shared state
   - `src/db/traits.rs` — RepositoryTrait (add new methods here)
   - `src/db/repository.rs` — PostgreSQL implementation
   - `src/db/mock.rs` — MockRepository for tests
   - `tests/integration.rs` — integration test pattern

4. **Write tests FIRST (TDD).** For each behavior:
   - Add unit tests in the module's `#[cfg(test)] mod tests`
   - Add integration tests in `tests/integration.rs` following existing patterns
   - Run `cargo test --features test-utils` to confirm tests fail (red)

5. **Implement.** Write the minimum code to make tests pass (green). Follow existing patterns:
   - Handlers: thin, delegate to domain/correlation modules
   - Config: `#[serde(default)]` for backward compatibility
   - Errors: use `PrefixdError` variants via `thiserror`
   - Logging: `tracing::info!`, `tracing::warn!`, `tracing::error!` with structured fields
   - Metrics: `Lazy<CounterVec>` / `Lazy<HistogramVec>` pattern from `src/observability/metrics.rs`

6. **Register new endpoints** if applicable:
   - Add `#[utoipa::path]` annotation on handler
   - Add route to `api_routes()` in `src/api/routes.rs`
   - Register types and paths in `src/api/openapi.rs`

7. **Update MockRepository** if you added new trait methods — add stubs that return empty/default results.

8. **Run full validation:**
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test --features test-utils
   ```
   Fix any failures before proceeding.

9. **Update documentation** if the feature description requires it:
   - `docs/api.md` for new endpoints
   - `docs/configuration.md` for new config fields
   - `docs/adr/` for architecture decisions (follow existing format: Context, Decision, Consequences)
   - `docs/adr/README.md` index
   - `CHANGELOG.md` Unreleased section
   - `AGENTS.md` test counts if changed

10. **Manual verification** — if the Docker stack is available, test with curl:
    ```bash
    curl -s http://localhost/v1/health
    curl -s -X POST http://localhost/v1/events -H 'Content-Type: application/json' -d '...'
    ```

11. **Commit** with a descriptive message following existing convention (`feat:`, `fix:`, `docs:`, `chore:`).

## Example Handoff

```json
{
  "salientSummary": "Implemented the correlation engine core module (src/correlation/) with time-windowed signal grouping, weighted confidence computation, and corroboration threshold checking. Added migration 007 creating signal_groups and signal_group_events tables. 14 unit tests cover grouping, weighting, confidence math, and edge cases. 4 integration tests cover the API endpoints. All pass, cargo clippy clean.",
  "whatWasImplemented": "src/correlation/mod.rs (CorrelationEngine with find_or_create_group, add_event, check_corroboration, compute_derived_confidence), src/correlation/config.rs (CorrelationConfig with per-source weights and per-playbook overrides), migrations/007_signal_groups.sql (signal_groups + signal_group_events tables, mitigations.signal_group_id column), src/db/traits.rs (4 new RepositoryTrait methods), src/db/repository.rs (PostgreSQL implementations), src/db/mock.rs (mock stubs)",
  "whatWasLeftUndone": "",
  "verification": {
    "commandsRun": [
      { "command": "cargo fmt --check", "exitCode": 0, "observation": "No formatting issues" },
      { "command": "cargo clippy -- -D warnings", "exitCode": 0, "observation": "No warnings" },
      { "command": "cargo test --features test-utils", "exitCode": 0, "observation": "140 unit + 48 integration + 9 postgres passed, 14 ignored" }
    ],
    "interactiveChecks": [
      { "action": "curl POST /v1/events with correlation enabled", "observed": "202 Accepted, signal group created, GET /v1/signal-groups returns one group" }
    ]
  },
  "tests": {
    "added": [
      { "file": "src/correlation/engine.rs", "cases": [
        { "name": "test_create_signal_group", "verifies": "New group created for novel (victim_ip, vector)" },
        { "name": "test_join_existing_group", "verifies": "Second event joins existing open group" },
        { "name": "test_weighted_confidence", "verifies": "Derived confidence = weighted average" }
      ]},
      { "file": "tests/integration.rs", "cases": [
        { "name": "test_signal_groups_list", "verifies": "GET /v1/signal-groups returns groups with pagination" },
        { "name": "test_signal_group_detail", "verifies": "GET /v1/signal-groups/{id} returns contributing events" }
      ]}
    ]
  },
  "discoveredIssues": []
}
```

## When to Return to Orchestrator

- Feature depends on a module or table that doesn't exist yet (check preconditions)
- Existing tests fail before your changes (pre-existing issue)
- Migration conflicts with existing schema
- Config changes would break backward compatibility in unexpected ways
- Requirements are ambiguous about correlation behavior edge cases
