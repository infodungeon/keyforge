-- Existing tables...
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    geometry_json TEXT NOT NULL,
    weights_json TEXT NOT NULL,
    pinned_keys TEXT NOT NULL,
    corpus_name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL,
    layout TEXT NOT NULL,
    score REAL NOT NULL,
    node_id TEXT NOT NULL,
    submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(job_id) REFERENCES jobs(id)
);

CREATE INDEX IF NOT EXISTS idx_results_score ON results(job_id, score ASC);

-- NEW: Community Submissions Table
CREATE TABLE IF NOT EXISTS submissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    layout_str TEXT NOT NULL,
    author TEXT DEFAULT 'Anonymous',
    notes TEXT,
    status TEXT DEFAULT 'pending', -- pending, approved, rejected
    submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP
);