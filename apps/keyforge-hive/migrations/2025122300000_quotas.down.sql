DROP INDEX IF EXISTS idx_jobs_owner_created;
ALTER TABLE users DROP COLUMN IF EXISTS max_daily_jobs;
ALTER TABLE users DROP COLUMN IF EXISTS max_active_jobs;
