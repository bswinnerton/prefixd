-- Migration 008: Partial unique index to prevent duplicate open signal groups
-- Ensures only one open signal group can exist per (victim_ip, vector) pair.
-- The CTE in find_or_create handles sequential races, but truly concurrent
-- inserts could bypass it. This index guarantees database-level uniqueness.

CREATE UNIQUE INDEX IF NOT EXISTS idx_signal_groups_open_unique
    ON signal_groups (victim_ip, vector) WHERE status = 'open';
