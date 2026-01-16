use keyforge_model::types::KeyCode;
use keyforge_physics::ScoringEngine;
use keyforge_model::{Layout, Corpus, Rubric};

fn main() {
    // Load SZR35 keyboard
    let kb_bytes = std::fs::read("data/system/keyboards/szr35.mpk.zst").unwrap();
    let kb = keyforge_model::Keyboard::decode(&kb_bytes).expect("Failed to decode keyboard");
    
    // Create a corpus that forces a choice for Space
    // Transition: A - Space - B
    // Case 1: All on Left hand
    // Case 2: Space on Right hand (alternation)
    let mut corpus = Corpus::default();
    corpus.char_freqs[b'A' as usize] = 100;
    corpus.char_freqs[b' ' as usize] = 100;
    corpus.char_freqs[b'B' as usize] = 100;
    corpus.bigrams.push((b'A' as u16, b' ' as u16, 100));
    corpus.bigrams.push((b' ' as u16, b'B' as u16, 100));
    corpus.trigrams.push((b'A' as u16, b' ' as u16, b'B' as u16, 100));

    let rubric = Rubric::default();
    let engine = ScoringEngine::new(&kb, &corpus, &rubric, &[]).unwrap();

    // SZR35 Colemak-DH like positions:
    // A: Left Home Pinky (idx 10)
    // B: Left Index Lower (idx 25) - actually let's use Index Home (idx 13)
    // Space_L: Thumb Left (idx 32)
    // Space_R: Thumb Right (idx 33)

    let run_test = |name: &str, layout: &Layout| {
        let report = engine.analyze(layout).unwrap();
        println!("{}: Distance = {:.4}, Travel/Key = {:.4}%", name, report.distance, report.travel_per_key * 100.0);
    };

    // Test 1: Space only on Left
    let mut keys_left = vec![KeyCode(0); 36];
    keys_left[10] = KeyCode(b'A' as u16);
    keys_left[13] = KeyCode(b'B' as u16);
    keys_left[32] = KeyCode(b' ' as u16);
    run_test("Left Space", &Layout::new_unchecked(keys_left));

    // Test 2: Space only on Right
    let mut keys_right = vec![KeyCode(0); 36];
    keys_right[10] = KeyCode(b'A' as u16);
    keys_right[13] = KeyCode(b'B' as u16);
    keys_right[33] = KeyCode(b' ' as u16);
    run_test("Right Space", &Layout::new_unchecked(keys_right));

    // Test 3: Both Spaces available
    let mut keys_both = vec![KeyCode(0); 36];
    keys_both[10] = KeyCode(b'A' as u16);
    keys_both[13] = KeyCode(b'B' as u16);
    keys_both[32] = KeyCode(b' ' as u16);
    keys_both[33] = KeyCode(b' ' as u16);
    run_test("Both Spaces", &Layout::new_unchecked(keys_both));
}
