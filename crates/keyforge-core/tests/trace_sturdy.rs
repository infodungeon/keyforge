use keyforge_core::config::Config;
use keyforge_core::geometry::KeyboardDefinition;
use keyforge_core::keycodes::KeycodeRegistry;
use keyforge_core::layouts::layout_string_to_u16; // FIXED Import
use keyforge_core::optimizer::mutation;
use keyforge_core::scorer::ScorerBuilder;
use std::path::PathBuf;

fn get_data_path(file: &str) -> String {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p.push("data");
    p.push(file);
    p.to_str().unwrap().to_string()
}

#[test]
fn trace_sturdy_scoring() {
    let cost_path = get_data_path("cost_matrix.csv");
    let ngrams_path = get_data_path("ngrams-all.tsv");
    let kb_path = get_data_path("keyboards/szr35.json");
    let weights_path = get_data_path("weights/ortho_split.json");
    let keycodes_path = get_data_path("keycodes.json");

    let mut config = Config::default();
    config.weights = keyforge_core::config::ScoringWeights::load_from_file(&weights_path);

    let registry = KeycodeRegistry::load_from_file(&keycodes_path).unwrap();
    let kb_def = KeyboardDefinition::load_from_file(&kb_path).unwrap();

    let scorer = ScorerBuilder::new()
        .with_weights(config.weights.clone())
        .with_defs(config.defs.clone())
        .with_geometry(kb_def.geometry.clone())
        .with_costs_from_file(&cost_path)
        .unwrap()
        .with_ngrams_from_file(&ngrams_path)
        .unwrap()
        .build()
        .unwrap();

    let layout_str = kb_def
        .layouts
        .get("Sturdy")
        .expect("Sturdy not found in JSON");
    println!("\n=== TRACING STURDY LAYOUT ===");
    println!("String: {}", layout_str);

    // FIXED: u16 logic
    let layout_codes = layout_string_to_u16(layout_str, scorer.key_count, &registry);
    let pos_map = mutation::build_pos_map(&layout_codes);

    println!("\n--- TOP 20 SFB PENALTIES ---");
    let mut sfb_hits = Vec::new();

    for &c1 in &scorer.active_chars {
        // active_chars are usize < 256
        let p1 = pos_map[c1];
        if p1 == 255 {
            continue;
        }

        let start = scorer.bigram_starts[c1];
        let end = scorer.bigram_starts[c1 + 1];

        for k in start..end {
            if scorer.bigrams_self_first[k] {
                let c2 = scorer.bigrams_others[k] as usize;
                let p2 = pos_map[c2];
                if p2 == 255 {
                    continue;
                }

                let freq = scorer.bigrams_freqs[k];
                let m = keyforge_core::scorer::physics::analyze_interaction(
                    &scorer.geometry,
                    p1 as usize,
                    p2 as usize,
                    &scorer.weights,
                );

                if m.is_same_hand && m.is_sfb {
                    let cost_res =
                        keyforge_core::scorer::costs::calculate_cost(&m, &scorer.weights);
                    let dist = keyforge_core::scorer::physics::get_geo_dist(
                        &scorer.geometry,
                        p1 as usize,
                        p2 as usize,
                        scorer.weights.weight_lateral_travel,
                        scorer.weights.weight_vertical_travel,
                    );

                    let total_penalty = (dist * cost_res.penalty_multiplier) * freq;

                    // FIXED: Cast back to u16 for lookup
                    let s1 = registry.get_label(c1 as u16);
                    let s2 = registry.get_label(c2 as u16);
                    sfb_hits.push((format!("{}->{}", s1, s2), total_penalty, freq));
                }
            }
        }
    }

    sfb_hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (label, cost, freq) in sfb_hits.iter().take(20) {
        println!("{:<8} | Cost: {:>8.0} | Freq: {:>8.0}", label, cost, freq);
    }

    // ... (rest of the file omitted for brevity, similar u16 casts required for Roll trace)
}
