fn run_validation(args: ValidateArgs, scorer: Arc<Scorer>) {
    println!("\n📊 === VALIDATION REPORT === 📊");
    println!(
        "{:<14} | {:<8} | {:<8} | {:<8} | {:<33} | {:<19} | {:<8}", 
        "Layout", "THEORY", "Travel", "Effort", "--- SFB BREAKDOWN ---", "--- MECHANICS ---", "FLOW"
    );
    println!(
        "{:<14} | {:<8} | {:<8} | {:<8} | {:<5} {:<5} {:<5} {:<5} {:<5} {:<5} | {:<5} {:<6} {:<6} | {:<8}", 
        "", "Total", "Dist", "Fing", "Base", "Lat", "WeakL", "Diag", "Long", "Bot", "SFR", "Lat", "Scis", "Cost"
    );
    println!("{:-<135}", "");

    // ... loop ...

    for (name, d) in results {
        println!(
            "{:<14} | {:<8.0} | {:<8.0} | {:<8.0} | {:<5.0} {:<5.0} {:<5.0} {:<5.0} {:<5.0} {:<5.0} | {:<5.0} {:<6.0} {:<6.0} | {:<8.0}", 
            name, 
            d.layout_score,
            d.geo_dist,
            d.finger_use,
            d.mech_sfb,
            d.mech_sfb_lat,
            d.mech_sfb_lat_weak, // NEW COLUMN
            d.mech_sfb_diag,
            d.mech_sfb_long,
            d.mech_sfb_bot,
            d.mech_sfr,
            d.mech_lat,
            d.mech_scis,
            d.flow_cost
        );
    }
    println!("{:-<135}", "");
}