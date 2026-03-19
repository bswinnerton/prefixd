---
name: ship
description: Release a new version of prefixd to GitHub
---

# /ship - prefixd Release Skill

Release a new version of prefixd to GitHub.

## Overview

This skill guides you through the complete release process interactively:
1. Pre-flight checks (fmt, clippy, tests, frontend build + tests)
2. Version bump and doc updates
3. Commit and tag
4. Push and monitor CI
5. Create GitHub release
6. Rebuild Docker containers
7. Post to related GitHub issues (with approval)

## Pre-flight Checks

Run these checks in parallel:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --features test-utils
cd frontend && bun run test && bun run build
```

**If `cargo fmt --check` fails:** Run `cargo fmt` to auto-fix, then re-check.

Also verify:
- Working tree is clean (`git status --porcelain` is empty)
- Version in `Cargo.toml` differs from latest git tag (`git describe --tags --abbrev=0`)
- No `Co-Authored-By` lines will be in the commit (user's global rule)

If any check fails, stop and help the user fix the issue before proceeding.

## Step 1: Version Bump

Determine new version (ask user if not specified). Bump version in `Cargo.toml`, then:

```bash
cargo check  # Updates Cargo.lock with new version
```

## Step 2: Update Docs

Update version strings across all docs:

| File | What to update |
|------|---------------|
| `Cargo.toml` | `version = "X.Y.Z"` |
| `CHANGELOG.md` | Rename `[Unreleased]` → `[X.Y.Z] - YYYY-MM-DD`, add comparison link |
| `ROADMAP.md` | `## Current Status: vX.Y.Z` |
| `README.md` | Version in health example, "Current version" line |
| `AGENTS.md` | `## Current State (vX.Y.Z)` |
| `docs/api.md` | Version strings in health response examples |
| `docs/deployment.md` | Version in health check example |

Search for stale version references:
```bash
rg "0\.OLD\.VERSION" --glob "*.md"
```

## Step 3: Commit and Tag

Show the user a summary of changes:
```bash
git diff --stat
```

Ask: **"Proceed with commit for version vX.Y.Z?"**

If approved:
```bash
git add -A
git commit -m "release: vX.Y.Z"
git tag vX.Y.Z
```

## Step 4: Push and Monitor CI

Ask: **"Push to origin and start CI?"**

If approved:
```bash
git push && git push origin vX.Y.Z
```

Then monitor CI:
```bash
gh run list --limit=1
gh run watch <run-id> --exit-status
```

If CI fails:
- Fetch logs: `gh run view <run-id> --log-failed`
- Help diagnose the issue
- Common fix: `cargo fmt` formatting differences
- If fix required: commit fix, delete old tag, retag, force push tag:
  ```bash
  git tag -d vX.Y.Z
  git push origin :refs/tags/vX.Y.Z
  # ... fix and commit ...
  git tag vX.Y.Z
  git push && git push origin vX.Y.Z
  ```

Wait for CI to complete successfully before proceeding.

## Step 5: Create GitHub Release

```bash
gh release create vX.Y.Z --title "vX.Y.Z" --notes "RELEASE_NOTES"
```

Release notes should summarize the CHANGELOG section for this version, organized by:
- What's New (features)
- Security (if any)
- Bug Fixes (if any)
- Full Changelog link

## Step 6: Rebuild Docker Containers

Ask: **"Rebuild local Docker containers with the new release?"**

If approved:
```bash
docker compose down
docker compose build --no-cache
docker compose up -d
```

Verify:
```bash
docker compose ps
curl -s http://localhost/v1/health | python3 -m json.tool
```

Confirm version in health response matches the new release.

## Step 7: Post-Release (GitHub Issues)

Check for related GitHub issues:
```bash
gh issue list --state open --limit 20
```

For each related issue:
1. Draft a response
2. Show the draft to the user
3. Ask: **"Post this response to issue #N?"**
4. Only post if explicitly approved

## Important Rules

- **No Co-Authored-By**: Never include Co-Authored-By lines in commits
- **Interactive**: Always ask before destructive/irreversible actions
- **CI must pass**: Never proceed past CI step if builds fail
- **All docs updated**: Don't skip the version string sweep
- **Frontend must build**: `bun run build` is a release gate, not optional
