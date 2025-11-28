use keyforge::scorer::ScoreDetails;
use keyforge::config::ScoringWeights;

// Helper to convert string to bytes for visualization
pub fn string_to_bytes(s: &str) -> [u8; 30] {
    let mut b = [32u8; 30]; // Default to space
    for (i, c) in s.bytes().take(30).enumerate() {
        b[i] = c;
    }
    b
}

pub fn print_layout_grid(name: &str, bytes: &[u8; 30]) {
    println!("\nLayout: {}", name);
    if bytes.len() < 30 {
        println!("(Invalid layout length)");
        return;
    }
    
    let rows = [
        &bytes[0..10],  // Top
        &bytes[10..20], // Home
        &bytes[20..30], // Bottom
    ];

    for row in rows {
        print!("  ");
        for &b in row {
            print!("{} ", b as char);
        }
        println!();
    }
}

pub fn print_scoring_report(results: &[(String, ScoreDetails)]) {
    println!("\n📊 === SCORING REPORT (Optimization Target) === 📊");
    println!(
        "{:<14} | {:>10} | {:>8} | {:>8} | {:^48} | {:^18} | {:^35}",
        "Layout", "TOTAL", "Travel", "Effort", "--- SFB COST ---", "--- MECH COST ---", "--- FLOW COST ---"
    );
    println!(
        "{:<14} | {:>10} | {:>8} | {:>8} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} | {:>5} {:>5} {:>5} | {:>7} {:>7} {:>7} {:>9}",
        "", "Score", "Dist", "Fing", "Base", "Lat", "WkL", "Diag", "Lng", "Bot", "SFR", "Lat", "Scis", "Redir", "Skip", "Roll", "Net"
    );
    println!("{:-<190}", "");

    for (name, d) in results {
        let roll_cost = d.flow_roll_in + d.flow_roll_out + d.flow_roll_tri;
        println!(
            "{:<14} | {:>10.0} | {:>8.0} | {:>8.0} | {:>7.0} {:>7.0} {:>7.0} {:>7.0} {:>7.0} {:>7.0} | {:>5.0} {:>5.0} {:>5.0} | {:>7.0} {:>7.0} {:>7.0} {:>9.0}",
            name, d.layout_score, d.geo_dist, d.finger_use,
            d.mech_sfb, d.mech_sfb_lat, d.mech_sfb_lat_weak, d.mech_sfb_diag, d.mech_sfb_long, d.mech_sfb_bot,
            d.mech_sfr, d.mech_lat, d.mech_scis,
            d.flow_redirect, d.flow_skip, roll_cost, d.flow_cost
        );
    }
}

pub fn print_statistical_report(results: &[(String, ScoreDetails)], w: &ScoringWeights) {
    println!("\n📈 === STATISTICAL ANALYSIS (Frequency %) === 📈");
    println!(
        "{:<14} | {:^53} | {:^26} | {:^39}",
        "Layout", "--- SFB BREAKDOWN (Weight) ---", "--- MECHANICS ---", "--- FLOW (IN / OUT) ---"
    );
    
    // Header Row
    println!(
        "{:<14} | {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} | {:<8} {:<8} {:<8} | {:<8} {:<8} {:<6} {:<6} {:<6} {:<6} {:<6}",
        "", 
        format!("Bas({:.0})", w.penalty_sfb_base),
        format!("Lat({:.0})", w.penalty_sfb_lateral),
        format!("WkL({:.0})", w.penalty_sfb_lateral_weak),
        format!("Dia({:.0})", w.penalty_sfb_diagonal),
        format!("Lng({:.0})", w.penalty_sfb_long),
        format!("Bot({:.0})", w.penalty_sfb_bottom),
        "SFR", 
        format!("Lat({:.0})", w.penalty_lateral),
        format!("Sci({:.0})", w.penalty_scissor),
        format!("Red({:.0})", w.penalty_redirect),
        format!("Skp({:.0})", w.penalty_skip),
        format!("R2I({:.0})", w.bonus_bigram_roll_in),
        format!("R2O({:.0})", w.bonus_bigram_roll_out),
        format!("R3I({:.0})", w.bonus_inward_roll),
        "R3O",
        "Pinky"
    );
    println!("{:-<190}", "");

    for (name, d) in results {
        let t_bi = if d.total_bigrams > 0.0 { d.total_bigrams } else { 1.0 };
        let t_tri = if d.total_trigrams > 0.0 { d.total_trigrams } else { 1.0 };
        let t_char = if d.total_chars > 0.0 { d.total_chars } else { 1.0 };

        println!(
            "{:<14} | {:<8.2} {:<8.2} {:<8.2} {:<8.2} {:<8.2} {:<8.2} | {:<8.2} {:<8.2} {:<8.2} | {:<8.2} {:<8.2} {:<6.2} {:<6.2} {:<6.2} {:<6.2} {:<6.2}",
            name, 
            (d.stat_sfb_base / t_bi) * 100.0, (d.stat_sfb_lat / t_bi) * 100.0, (d.stat_sfb_lat_weak / t_bi) * 100.0,
            (d.stat_sfb_diag / t_bi) * 100.0, (d.stat_sfb_long / t_bi) * 100.0, (d.stat_sfb_bot / t_bi) * 100.0,
            (d.stat_sfr / t_bi) * 100.0, (d.stat_lat / t_bi) * 100.0, (d.stat_scis / t_bi) * 100.0,
            (d.stat_redir / t_tri) * 100.0, (d.stat_skip / t_tri) * 100.0, 
            (d.stat_roll_in / t_bi) * 100.0, (d.stat_roll_out / t_bi) * 100.0, 
            (d.stat_roll3_in / t_tri) * 100.0, (d.stat_roll3_out / t_tri) * 100.0,
            (d.stat_pinky_reach / t_char) * 100.0
        );
    }
    println!("{:-<190}", "");
}

pub fn print_comparison_report(results: &[(String, ScoreDetails)]) {
    use std::collections::HashMap;
    
    struct RefStats { sfb: f32, lsb: f32, scis: f32, roll: f32, redir: f32, pinky: f32 }
    let mut ref_stats = HashMap::new();
    ref_stats.insert("qwerty".to_string(), RefStats { sfb: 4.38, lsb: 4.55, scis: 1.46, roll: 40.76, redir: 6.22, pinky: 2.47 });
    ref_stats.insert("dvorak".to_string(), RefStats { sfb: 1.87, lsb: 0.80, scis: 0.08, roll: 39.20, redir: 1.55, pinky: 4.13 });
    ref_stats.insert("colemak".to_string(), RefStats { sfb: 1.70, lsb: 2.26, scis: 0.26, roll: 49.20, redir: 5.33, pinky: 0.78 });
    ref_stats.insert("colemak_dh".to_string(), RefStats { sfb: 0.91, lsb: 1.27, scis: 0.15, roll: 49.20, redir: 5.33, pinky: 0.78 });
    ref_stats.insert("workman".to_string(), RefStats { sfb: 1.97, lsb: 1.11, scis: 0.47, roll: 47.40, redir: 6.05, pinky: 0.78 });
    ref_stats.insert("engram".to_string(), RefStats { sfb: 1.01, lsb: 0.41, scis: 0.36, roll: 44.32, redir: 2.27, pinky: 5.71 });
    ref_stats.insert("canary".to_string(), RefStats { sfb: 0.66, lsb: 1.75, scis: 0.42, roll: 50.36, redir: 3.39, pinky: 2.96 });
    ref_stats.insert("gallium".to_string(), RefStats { sfb: 0.64, lsb: 1.00, scis: 0.95, roll: 46.07, redir: 1.87, pinky: 3.16 });
    ref_stats.insert("graphite".to_string(), RefStats { sfb: 0.68, lsb: 0.87, scis: 0.41, roll: 46.01, redir: 1.80, pinky: 2.34 });

    println!("\n🧐 === REALITY CHECK (Calculated vs Reference %) === 🧐");
    println!("{:<16} | {:^13} | {:^13} | {:^13} | {:^13} | {:^13} | {:^13}", "Layout", "SFB", "LSB", "Scissor", "Rolls", "Redir", "Pinky");
    println!("{:<16} | {:^6} {:^6} | {:^6} {:^6} | {:^6} {:^6} | {:^6} {:^6} | {:^6} {:^6} | {:^6} {:^6}", "", "Ref", "Calc", "Ref", "Calc", "Ref", "Calc", "Ref", "Calc", "Ref", "Calc", "Ref", "Calc");
    println!("{:-<130}", "");

    for (name, d) in results {
        if !ref_stats.contains_key(name) { continue; }
        let t_bi = if d.total_bigrams > 0.0 { d.total_bigrams } else { 1.0 };
        let t_tri = if d.total_trigrams > 0.0 { d.total_trigrams } else { 1.0 };
        let t_char = if d.total_chars > 0.0 { d.total_chars } else { 1.0 };

        let sfb = (d.stat_sfb / t_bi) * 100.0;
        let lsb = ((d.stat_lsb + d.stat_sfb_lat + d.stat_sfb_lat_weak) / t_bi) * 100.0;
        let scis = (d.stat_scis / t_bi) * 100.0;
        let roll = (d.stat_roll / t_bi) * 100.0; 
        let redir = (d.stat_redir / t_tri) * 100.0;
        let pinky = (d.stat_pinky_reach / t_char) * 100.0;

        let r = ref_stats.get(name).unwrap();
        println!("{:<16} | {:5.2} {:5.2} | {:5.2} {:5.2} | {:5.2} {:5.2} | {:5.2} {:5.2} | {:5.2} {:5.2} | {:5.2} {:5.2}",
            name, r.sfb, sfb, r.lsb, lsb, r.scis, scis, r.roll, roll, r.redir, redir, r.pinky, pinky);
    }
    println!("{:-<130}", "");
}