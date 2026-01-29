---
name: SQL Optimization Oracle
description: Enforces high-performance database patterns and projection-first repository logic.
version: 1.0.0
---

# SQL Optimization Oracle

Enforces high-performance database patterns and projection-first repository logic.

## Instructions

### 1. Zero-Leak Projections
- NEVER use `SELECT *` in repository handlers.
- ALWAYS use the `Projection` trait to map SQL results to domain-specific bundles.
- Ensure SQL queries only fetch the exact columns required by the projection.

### 2. N+1 Elimination
- Identify loops that execute queries (N+1 anti-pattern).
- Replace with `JOIN` or `IN` clause batch fetching.
- For complex relationships, use `LATERAL JOIN` or `JSONB` aggregations to fetch tree structures in a single round-trip.

### 3. Index Strategy
- Every query must be supported by an index.
- Use `GIN` indexes for JSONB metadata fields.
- Use `BRIN` for large, time-ordered log/event tables.
- Validate every migration with `EXPLAIN (ANALYZE, BUFFERS)`.

### 4. Statement Performance
- Prefer `EXISTS` over `COUNT(*) > 0` for existence checks.
- Avoid `OFFSET` for pagination; use keyset pagination (seek-method).
- Use CTEs (`WITH`) sparingly for clarity, but be aware of materialization boundaries in older Postgres versions.

## Verification
- Run `just db-lint` to verify schema consistency.
- Inspect execution plans for any query affecting >10k rows.