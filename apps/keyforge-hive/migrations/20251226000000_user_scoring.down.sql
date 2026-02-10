DROP INDEX IF EXISTS idx_jobs_reaper;
ALTER TABLE jobs DROP COLUMN IF EXISTS is_pinned;
ALTER TABLE users DROP COLUMN IF EXISTS contribution_count;
DROP INDEX IF EXISTS idx_results_user;
ALTER TABLE results DROP COLUMN IF EXISTS user_id;
