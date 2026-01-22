pub const CLAIM_JOB_QUERY: &str = r"
            WITH locked_job AS (
                SELECT id 
                FROM jobs 
                WHERE status = 'active' 
                ORDER BY priority DESC, created_at ASC 
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            ),
            updated_job AS (
                UPDATE jobs
                SET status = 'processing',
                    started_at = CURRENT_TIMESTAMP
                FROM locked_job
                WHERE jobs.id = locked_job.id
                RETURNING jobs.*
            )
            SELECT 
                u.id,
                jsonb_build_object(
                    'meta', jsonb_build_object('name', k.name, 'author', k.author, 'version', k.version, 'notes', k.notes, 'type', k.kb_type),
                    'geometry', jsonb_build_object(
                        'keys', (SELECT jsonb_agg(jsonb_build_object('id', 'k'||kk.idx, 'hand', kk.hand, 'finger', kk.finger, 'row', kk.row_idx, 'col', kk.col_idx, 'x', kk.x, 'y', kk.y, 'w', kk.w, 'h', kk.h, 'is_stretch', kk.is_stretch, 'r', kk.r) ORDER BY kk.idx) FROM keyboard_keys kk WHERE kk.keyboard_id = k.id),
                        'prime_slots', (SELECT coalesce(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_prime),
                        'med_slots', (SELECT coalesce(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_med),
                        'low_slots', (SELECT coalesce(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_low),
                        'home_row', k.home_row
                    )
                ) as geometry_json,
                sp.weights as weights_json,
                (to_jsonb(sc) - 'id' - 'config_hash') as params_json,
                u.pinned_keys, 
                u.corpus_name, 
                u.cost_matrix,
                u.parent_job_id,
                u.priority
            FROM updated_job u
            JOIN keyboards k ON u.keyboard_id = k.id
            JOIN scoring_profiles sp ON u.scoring_profile_id = sp.id
            JOIN search_configs sc ON u.search_config_id = sc.id
            ";

pub const GET_JOB_CONFIG_QUERY: &str = r"
            SELECT 
                jsonb_build_object(
                    'keys', (SELECT COALESCE(jsonb_agg(jsonb_build_object('x', kk.x, 'y', kk.y, 'w', kk.w, 'h', kk.h, 'row', kk.row_idx, 'col', kk.col_idx, 'hand', kk.hand, 'finger', kk.finger, 'is_stretch', kk.is_stretch, 'id', 'k' || kk.idx, 'r', kk.r) ORDER BY kk.idx), '[]'::jsonb) FROM keyboard_keys kk WHERE kk.keyboard_id = k.id),
                    'prime_slots', (SELECT COALESCE(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_prime),
                    'med_slots', (SELECT COALESCE(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_med),
                    'low_slots', (SELECT COALESCE(jsonb_agg(idx), '[]'::jsonb) FROM keyboard_keys WHERE keyboard_id = k.id AND is_low),
                    'home_row', k.home_row
                ) as geometry_json,
                sp.weights as weights_json,
                j.corpus_name,
                j.cost_matrix
            FROM jobs j
            JOIN keyboards k ON j.keyboard_id = k.id
            JOIN scoring_profiles sp ON j.scoring_profile_id = sp.id
            WHERE j.id = $1
            ";

pub const INSERT_JOB_QUERY: &str = r"
            INSERT INTO jobs (
                id, keyboard_id, scoring_profile_id, search_config_id, 
                pinned_keys, corpus_name, cost_matrix, owner_id, 
                parent_job_id, priority
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO NOTHING
            ";

pub const INSERT_KEYBOARD_QUERY: &str = r"
            INSERT INTO keyboards (name, author, version, notes, kb_type, home_row, unique_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (unique_hash) DO UPDATE SET created_at = CURRENT_TIMESTAMP
            RETURNING id
            ";

pub const INSERT_KEY_QUERY: &str = r"
            INSERT INTO keyboard_keys 
            (keyboard_id, idx, x, y, w, h, hand, finger, row_idx, col_idx, is_stretch, is_prime, is_med, is_low, r)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ";

pub const PRUNE_STALE_JOBS_WITH_NODE: &str = r"
            UPDATE jobs
            SET 
                status = CASE 
                    WHEN retry_count >= $2 THEN 'failed' 
                    ELSE 'active' 
                END,
                node_id = NULL,
                started_at = NULL,
                retry_count = retry_count + 1
            WHERE 
                status = 'processing' 
                AND (
                    (node_id IS NULL AND started_at < NOW() - make_interval(mins => $1))
                    OR node_id IN (SELECT id FROM nodes WHERE last_seen < NOW() - make_interval(mins => $1))
                )
        ";
