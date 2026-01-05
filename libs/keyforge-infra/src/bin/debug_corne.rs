use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // Point to the sandbox where the failing asset lives
    let root = PathBuf::from("sandbox/client");
    println!("�� Inspecting 'corne' in: {:?}", root);
    
    let provider = FsProvider::new(root);
    
    match provider.load_keyboard("corne").await {
        Ok(kb) => {
             println!("✅ Successfully Loaded Corne");
             println!("   Name: {}", kb.meta.name);
             println!("   Key Count: {}", kb.geometry.keys.len());
             if let Some(k) = kb.geometry.keys.first() {
                 println!("   First Key: {:?}", k);
                 // Explicitly check for zero-width/height which causes Infinity SVG errors
                 // Assuming w/h or width/height fields exist (will confirm from cat output)
             }
        },
        Err(e) => {
            println!("❌ Load Failed: {:?}", e);
            println!("   (This matches the UI 'Failed to load reference' error)");
        }
    }
}
