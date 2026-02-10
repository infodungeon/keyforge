use keyforge_adapter::loader::AssetLoader;
use keyforge_boundary::SafePath;
use keyforge_infra::asset::fs_provider::FsProvider;
use keyforge_model::geometry::KeyboardDefinition;
use tokio::fs;

#[tokio::test]
async fn content_hash_fingerprint_uniqueness() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let root = SafePath::from_trusted_root_path(temp.path().to_path_buf());

    // Setup FsProvider
    let provider = FsProvider::new(root.clone());

    // Create two files with different content
    let dir = temp.path().join("system/keyboards");
    fs::create_dir_all(&dir).await?;

    // ...
    let kb1 = KeyboardDefinitionDto::from(KeyboardDefinition::default());
    let json1 = serde_json::to_string(&kb1)?;
    let path1 = dir.join("kb1.json");
    fs::write(&path1, &json1).await?;

    let mut kb2_def = KeyboardDefinition::default();
    kb2_def.meta.name = "Modified".to_string();
    let kb2 = KeyboardDefinitionDto::from(kb2_def);
    let json2 = serde_json::to_string(&kb2)?;
    let path2 = dir.join("kb2.json");
    fs::write(&path2, &json2).await?;

    //    use keyforge_protocol::KeyboardDefinitionDto;
    // ...
    // let asset1 = provider.load::<keyforge_protocol::KeyboardDefinitionDto>("kb1.json").await?;
    // let asset2 = provider.load::<keyforge_protocol::KeyboardDefinitionDto>("kb2.json").await?;

    use keyforge_protocol::KeyboardDefinitionDto;

    // ...
    // Load and check hashes
    let asset1 = provider.load::<KeyboardDefinitionDto>("kb1.json").await?;
    let asset2 = provider.load::<KeyboardDefinitionDto>("kb2.json").await?;

    assert_ne!(
        asset1.content_hash, asset2.content_hash,
        "Hashes must differ"
    );

    // Check stability
    let asset1_again = provider.load::<KeyboardDefinitionDto>("kb1.json").await?;
    assert_eq!(
        asset1.content_hash, asset1_again.content_hash,
        "Hash must be stable"
    );

    Ok(())
}
