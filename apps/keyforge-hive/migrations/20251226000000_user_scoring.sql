-- 1. Link Results to Users
-- This allows us to see who found a specific layout.
-- It is nullable to support Anonymous contributions.
ALTER TABLE results 
ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_results_user ON results(user_id);

-- 2. The Scoreboard (Permanent Stats)
-- This counter increments forever, even if the actual result rows are deleted 
-- by the "Top 5" reaper later. This preserves the user's "Fame".
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS contribution_count BIGINT DEFAULT 0;

-- 3. Storage Management (Pinning)
-- Jobs are normally deleted after 7 days if they aren't pinned.
-- Pinned jobs (and their Top 5 results) are kept forever.
ALTER TABLE jobs 
ADD COLUMN IF NOT EXISTS is_pinned BOOLEAN DEFAULT FALSE;

-- Index to help the Reaper quickly find old, unpinned jobs to delete
CREATE INDEX IF NOT EXISTS idx_jobs_reaper ON jobs(created_at) WHERE is_pinned = FALSE;