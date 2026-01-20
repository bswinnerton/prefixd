-- Add raw_details and action columns for detector integration
ALTER TABLE events ADD COLUMN IF NOT EXISTS raw_details JSONB;
ALTER TABLE events ADD COLUMN IF NOT EXISTS action TEXT NOT NULL DEFAULT 'ban';

-- Index for unban lookups (find event by source + external_event_id)
CREATE INDEX IF NOT EXISTS idx_events_source_external ON events(source, external_event_id) WHERE external_event_id IS NOT NULL;
