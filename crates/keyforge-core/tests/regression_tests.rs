use keyforge_core::api::{load_dataset, validate_layout, KeyForgeState};
use std::path::PathBuf;
use std::thread;

fn get_workspace_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    if cwd.join("data").exists() {
        return cwd;
    }
    let up_two = cwd.join("../../");
    if up_two.join("data").exists() {
        return up_two;
    }
    panic!("Could not locate 'data' directory. CWD: {:?}", cwd);
}

#[test]
fn test_regression_qwerty_vs_colemak_api() {
    let root = get_workspace_root();
    let data_dir = root.join("data");

    // 1. HARD ASSERTION: Verify Data Existence
    let required_files = [
        "ngrams-all.tsv",
        "cost_matrix.csv",
        "keyboards/ortho_30.json",
        "weights/ortho_split.json",
    ];

    for file in required_files {
        let p = data_dir.join(file);
        if !p.exists() {
            panic!(
                "\n❌ REGRESSION TEST FAILURE: Missing Data File\nFile: {:?}\n",
                p
            );
        }
    }

    let builder = thread::Builder::new().stack_size(8 * 1024 * 1024);

    let handler = builder
        .spawn(move || {
            let state = KeyForgeState::default();
            let session_id = "regression_test";

            // Load Dataset with absolute paths
            load_dataset(
                &state,
                session_id,
                data_dir.join("cost_matrix.csv").to_str().unwrap(),
                data_dir.join("ngrams-all.tsv").to_str().unwrap(),
                &Some(
                    data_dir
                        .join("keyboards/ortho_30.json")
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
                None,
                None,
            )
            .expect("Failed to load dataset via API");

            // Fetch layout strings
            let (qwerty_str, colemak_str) = {
                // FIXED: .lock() -> .read()
                let sessions = state.sessions.read().unwrap();
                let session = sessions
                    .get(session_id)
                    .expect("Session not found in state");

                let kb = &session.kb_def;
                let q = kb
                    .layouts
                    .get("Qwerty")
                    .expect("Layout 'Qwerty' missing")
                    .clone();
                let c = kb
                    .layouts
                    .get("Colemak")
                    .expect("Layout 'Colemak' missing")
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

            assert!(
                score_q > score_c,
                "Regression: Qwerty ({}) should be worse than Colemak ({})",
                score_q,
                score_c
            );

            assert!(
                res_qwerty.score.mech_sfb > 500.0,
                "Regression: Qwerty SFB score ({}) is suspiciously low",
                res_qwerty.score.mech_sfb
            );
        })
        .unwrap();

    handler.join().unwrap();
}
