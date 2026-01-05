use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use std::path::PathBuf;

#[tokio::test]
#[ignore]
async fn inspect_szr35_geometry() {
    // Setup Provider pointing to sandbox assets
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap() // libs
        .parent().unwrap() // root
        .join("data");
    
    let provider = FsProvider::new(root);
    let kb = provider.load_keyboard("szr35").await.expect("Failed to load szr35");

    println!("\nSZR35 GEOMETRY INSPECTION");
    println!("Total Keys: {}", kb.geometry.keys.len());
    println!("{:<6} | {:<4} | {:<6} | {:<4} | {:<4} | {:<6} | {:<6}", 
        "INDEX", "HAND", "FINGER", "ROW", "COL", "X", "Y");
    println!("{:-<50}", "-");

    // Collect and sort for readable grid output
    let mut keys = kb.geometry.keys.clone();
    keys.sort_by(|a, b| {
        a.hand.as_u8().cmp(&b.hand.as_u8())
            .then(a.row.0.cmp(&b.row.0))
            .then(a.col.0.cmp(&b.col.0))
    });

    for k in keys {
        println!("{:<6} | {:<4} | {:<6} | {:<4} | {:<4} | {:<6.2} | {:<6.2}", 
            k.index, 
            k.hand.as_u8(), 
            k.finger.as_u8(), 
            k.row.0, 
            k.col.0, 
            k.x, 
            k.y
        );
    }
    println!("\n");
}
