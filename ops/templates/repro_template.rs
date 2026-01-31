//! KeyForge Reproduction Harness
//! Usage: rustc repro.rs && ./repro
//! Goal: Isolate logic bugs without compiling the full workspace.

use std::collections::HashMap;

// === MOCKS (The Shell) ===
// Paste minimal structs here to simulate the environment.

#[derive(Debug, Clone, PartialEq)]
struct KeyIndex(usize);

#[derive(Debug, Clone, PartialEq)]
struct Score(i64);

// === LOGIC UNDER TEST (The Nucleus) ===
// Paste the suspicious function here.

fn calculate_something(idx: KeyIndex) -> Score {
    Score((idx.raw() * 10) as i64)
}

// === VERIFICATION (The Test) ===

fn main() {
    println!("Running Reproduction Harness...");

    let input = KeyIndex::new(5);
    let expected = Score(50);
    let actual = calculate_something(input);

    if actual == expected {
        println!("✅ PASS: Logic behaves as expected.");
    } else {
        println!("❌ FAIL: Expected {:?}, got {:?}", expected, actual);
        std::process::exit(1);
    }
}
