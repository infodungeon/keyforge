// UPDATED: use keyforge_core
use keyforge_core::api::{load_dataset, validate_layout, KeyForgeState};
use std::path::Path;
use std::thread;

fn has_real_data() -> bool {
    Path::new("data/ngrams-all.tsv").exists()
        && Path::new("data/cost_matrix.csv").exists()
        && Path::new("data/keyboards/ortho_30.json").exists()
        && Path::new("data/weights/ortho_split.json").exists()
}

#[test]
fn test_regression_qwerty_vs_colemak_api() {
    if !has_real_data() {
        println!("Skipping regression test: Real data not found");
        return;
    }

    let builder = thread::Builder::new().stack_size(8 * 1024 * 1024);

    let handler = builder
        .spawn(|| {
            let state = KeyForgeState::default();
            let session_id = "regression_test";

            let load_result = load_dataset(
                &state,
                session_id,
                "data/cost_matrix.csv",
                "data/ngrams-all.tsv",
                &Some("data/keyboards/ortho_30.json".to_string()),
                None,
                None,
            );

            assert!(
                load_result.is_ok(),
                "Failed to load dataset via API: {:?}",
                load_result.err()
            );

            // Fetch strings
            let (qwerty_str, colemak_str) = {
                let sessions = state.sessions.lock().unwrap();
                let session = sessions.get(session_id).expect("Session not found");

                let kb = &session.kb_def;
                let q = kb
                    .layouts
                    .get("Qwerty")
                    .expect("Qwerty layout missing")
                    .clone();
                let c = kb
                    .layouts
                    .get("Colemak")
                    .expect("Colemak layout missing")
                    .clone();
                (q, c)
            };

            // Get Scores
            let res_qwerty = validate_layout(&state, session_id, qwerty_str, None)
                .expect("Qwerty validation failed");
            let res_colemak = validate_layout(&state, session_id, colemak_str, None)
                .expect("Colemak validation failed");

            // Assertions
            let score_q = res_qwerty.score.layout_score;
            let score_c = res_colemak.score.layout_score;

            println!(
                "API Real Scores -> QWERTY: {:.0}, Colemak: {:.0}",
                score_q, score_c
            );

            assert!(score_q > 0.0);
            assert!(score_c > 0.0);
            assert!(score_q > score_c);
            assert!(res_qwerty.score.mech_sfb > 1000.0);
        })
        .unwrap();

    handler.join().unwrap();
}
