// ===== keyforge/tests/regression_tests.rs =====
use keyforge::api::{load_dataset, validate_layout, KeyForgeState};
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

    // Spawn a thread with a larger stack (8MB) to prevent overflow in Debug builds
    // caused by the large [f32; 256*256] matrix in Scorer.
    let builder = thread::Builder::new().stack_size(8 * 1024 * 1024);

    let handler = builder
        .spawn(|| {
            // 1. Initialize State
            let state = KeyForgeState::default();

            let load_result = load_dataset(
                &state,
                "data/cost_matrix.csv",
                "data/ngrams-all.tsv",
                &Some("data/keyboards/ortho_30.json".to_string()),
                None,
            );

            assert!(
                load_result.is_ok(),
                "Failed to load dataset via API: {:?}",
                load_result.err()
            );

            // 2. Fetch Layout Strings (Owned)
            // We use a scope block to ensure the lock is dropped immediately
            let (qwerty_str, colemak_str) = {
                let guard = state.kb_def.lock().unwrap();
                let kb = guard.as_ref().expect("Keyboard definition not loaded");

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
            }; // Lock is released here

            // 3. Get Scores
            let res_qwerty =
                validate_layout(&state, qwerty_str, None).expect("Qwerty validation failed");
            let res_colemak =
                validate_layout(&state, colemak_str, None).expect("Colemak validation failed");

            // 4. Assertions
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
                "QWERTY ({:.0}) should be worse (higher) than Colemak ({:.0})",
                score_q,
                score_c
            );

            // Verify weights loaded (Ortho Split should have high SFB penalty ~400)
            // QWERTY has significant SFBs, so mechanical SFB score should be high.
            assert!(
                res_qwerty.score.mech_sfb > 1000.0,
                "SFB Score too low, weights might not have loaded."
            );
        })
        .unwrap();

    handler.join().unwrap();
}
