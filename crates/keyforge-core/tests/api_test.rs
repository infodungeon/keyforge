// UPDATED: use keyforge_core
use keyforge_core::api::{KeyForgeState, load_dataset, validate_layout};
use std::fs::File;
use std::io::Write;

#[test]
fn test_api_integration_and_serialization() {
    // Spawn a thread with a larger stack (8MB) to handle Scorer allocation
    let builder = std::thread::Builder::new().stack_size(8 * 1024 * 1024);

    let handler = builder.spawn(|| {
        // 1. Setup Fake Data
        let _dir = tempfile::tempdir().unwrap(); 
        let cost_path = _dir.path().join("api_cost.csv");
        let ngram_path = _dir.path().join("api_ngrams.tsv");
        let geo_path = _dir.path().join("api_kb.json"); 

        // Scope block to ensure files are closed/flushed before loading
        {
            let mut cost_file = File::create(&cost_path).unwrap();
            writeln!(cost_file, "From,To,Cost\nKeyQ,KeyW,10.0").unwrap();

            let mut ngram_file = File::create(&ngram_path).unwrap();
            writeln!(ngram_file, "q\t1000").unwrap(); 
            writeln!(ngram_file, "w\t1000").unwrap(); 
            writeln!(ngram_file, "qw\t1000").unwrap(); 

            let mut json = String::from(r#"{
                "meta": {
                    "name": "TestBoard",
                    "author": "Test",
                    "version": "1.0"
                },
                "geometry": {
                    "keys": ["#);

            for i in 0..30 {
                let s = format!(
                    r#"{{"hand": 0, "finger": {}, "row": 0, "col": {}, "x": {}, "y": 0.0, "is_stretch": false}}"#,
                    i % 5, i, i as f32
                );
                json.push_str(&s);
                if i < 29 { json.push(','); }
            }
            
            json.push_str(r#"], 
                    "prime_slots": [], 
                    "med_slots": [], 
                    "low_slots": [],
                    "home_row": 1
                },
                "layouts": {} 
            }"#);

            let mut geo_file = File::create(&geo_path).unwrap();
            writeln!(geo_file, "{}", json).unwrap();
        } 

        // 2. Initialize State
        let state = KeyForgeState::default();
        let session_id = "test_session"; 

        let load_res = load_dataset(
            &state, 
            session_id, 
            cost_path.to_str().unwrap(), 
            ngram_path.to_str().unwrap(), 
            &Some(geo_path.to_str().unwrap().to_string()), 
            Some(1.0),
            None 
        );
        
        if let Err(e) = &load_res {
            println!("Load failed: {}", e);
        }
        assert!(load_res.is_ok(), "Failed to load dataset");

        // 3. Validate "QW..." (Uppercase Input from UI)
        let layout = "QW".to_string() + "ABCDEFGHIJKLMONPRSTUVXYZ1234"; // 30 chars
        
        let res = validate_layout(&state, session_id, layout, None).expect("Validation failed");

        // 4. CHECK: Did we get scores?
        assert!(res.score.total_chars > 0.0, "Total chars is 0.");
        assert!(res.score.total_bigrams > 0.0, "Total bigrams is 0.");

        // 5. CHECK: Heatmap
        assert_eq!(res.heatmap.len(), 30);
        assert!(res.heatmap[0] > 0.0, "Heatmap index 0 (Q) is cold. Should be hot.");
    }).unwrap();

    handler.join().unwrap();
}