-- Add home_row column to keyboards table
ALTER TABLE keyboards ADD COLUMN IF NOT EXISTS home_row INTEGER DEFAULT 1;
