-- Add quota columns to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS max_active_jobs INTEGER DEFAULT 5;
ALTER TABLE users ADD COLUMN IF NOT EXISTS max_daily_jobs INTEGER DEFAULT 50;

-- Create index for quota checks
CREATE INDEX IF NOT EXISTS idx_jobs_owner_created ON jobs(owner_id, created_at);
