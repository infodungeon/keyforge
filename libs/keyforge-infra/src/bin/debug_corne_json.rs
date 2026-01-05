use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let root = PathBuf::from("sandbox/client");
    let provider = FsProvider::new(root);
    
    if let Ok(kb) = provider.load_keyboard("corne").await {
        // Mimic the backend command serialization
        let json = serde_json::to_string_pretty(&kb.geometry).unwrap();
        println!("{}", json);
    }
}
