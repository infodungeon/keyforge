-- Add raw_score column to results table to preserve absolute precision
ALTER TABLE results ADD COLUMN IF NOT EXISTS raw_score BIGINT DEFAULT 0;

-- Backfill existing results (approximate from float score)
UPDATE results SET raw_score = (score * 1000000.0)::BIGINT WHERE raw_score = 0;
