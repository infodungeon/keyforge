-- Optimize job queue fetching
CREATE INDEX IF NOT EXISTS idx_jobs_fetch ON jobs (status, priority DESC, created_at ASC);
