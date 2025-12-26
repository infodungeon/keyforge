-- Add tracking columns for job liveness
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS started_at TIMESTAMPTZ;
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS retry_count INTEGER DEFAULT 0;

-- Index for the Reaper to find stale jobs quickly
CREATE INDEX IF NOT EXISTS idx_jobs_stale ON jobs(status, started_at) WHERE status = 'processing';