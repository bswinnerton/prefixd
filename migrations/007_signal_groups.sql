-- Migration 007: Signal groups for multi-signal correlation
-- Creates tables for grouping related attack events by (victim_ip, vector)
-- within configurable time windows for corroboration-based mitigation decisions.

CREATE TABLE IF NOT EXISTS signal_groups (
    group_id        UUID PRIMARY KEY,
    victim_ip       TEXT NOT NULL,
    vector          TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    window_expires_at TIMESTAMPTZ NOT NULL,
    derived_confidence REAL NOT NULL DEFAULT 0.0,
    source_count    INTEGER NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'open',
    corroboration_met BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS signal_group_events (
    group_id      UUID NOT NULL REFERENCES signal_groups(group_id),
    event_id      UUID NOT NULL,
    source_weight REAL NOT NULL DEFAULT 1.0,
    PRIMARY KEY (group_id, event_id)
);

-- Nullable FK from mitigations to the signal group that triggered them
ALTER TABLE mitigations ADD COLUMN IF NOT EXISTS signal_group_id UUID REFERENCES signal_groups(group_id);

-- Index for looking up open groups by (victim_ip, vector)
CREATE INDEX IF NOT EXISTS idx_signal_groups_victim_vector_status
    ON signal_groups (victim_ip, vector, status);

-- Index for expiry sweep: find open groups past their window
CREATE INDEX IF NOT EXISTS idx_signal_groups_status_expires
    ON signal_groups (status, window_expires_at);
