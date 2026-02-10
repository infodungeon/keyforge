use keyforge_adapter::loader::AssetLoader;
use keyforge_boundary::SafePath;
use keyforge_compute::use_cases::OptimizationUseCase;
use keyforge_infra::asset::fs_provider::FsProvider;
use keyforge_protocol::{CostMatrixSourceDto, JobRequest};
use tokio::fs;

#[tokio::test]
async fn cache_invalidation_on_content_update() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let root = SafePath::from_trusted_root_path(temp.path().to_path_buf());
    let provider = FsProvider::new(root);

    // Create necessary dirs
    fs::create_dir_all(temp.path().join("system/weights")).await?;
    fs::create_dir_all(temp.path().join("system/keyboards")).await?;
    fs::create_dir_all(temp.path().join("system/corpora")).await?;

    // Create dummy assets
    // Keyboard
    fs::write(temp.path().join("system/keyboards/default.json"), r#"{
        "meta": {"name": "default", "author": "", "version": "", "notes": "", "kb_type": "standard"},
        "geometry": {
            "keys": [{"index": 0, "label": "A", "x": 0.0, "y": 0.0, "hand": 0, "finger": 1, "row": 0, "col": 0, "w": 1.0, "h": 1.0, "is_home": true, "is_stretch": false, "r": 0.0, "rx": 0.0, "ry": 0.0}],
            "prime_slots": [0], "med_slots": [], "low_slots": [], "home_row": 0
        },
        "layouts": {}
    }"#).await?;

    // Cost Matrix A
    fs::write(
        temp.path().join("system/weights/matrix.json"),
        r#"{
        "meta": {"version": "1.0", "description": "test", "unit": "pts"},
        "models": {
            "model_a_row_staggered": {
                "description": "test",
                "static_costs": {
                    "universal_hand": {
                        "fingers": {
                            "index": {"base": {"r0": 100, "r1": 100, "r2": 100, "r3": 100}},
                            "middle": {"base": {"r0": 100, "r1": 100, "r2": 100, "r3": 100}},
                            "ring": {"base": {"r0": 100, "r1": 100, "r2": 100, "r3": 100}},
                            "pinky": {"base": {"r0": 100, "r1": 100, "r2": 100, "r3": 100}},
                            "thumb": {"pos_1": 100, "pos_2": 100, "pos_3": 100}
                        }
                    }
                }
            }
        },
        "dynamic_rules": {"sequence_modifiers": {}, "penalties": {}, "constraints": {}}
    }"#,
    )
    .await?;

    // Corpus (default is en_small.json)
    fs::write(temp.path().join("system/corpora/en_small.json"), r#"{
        "char_freqs": [], "bigrams": [], "trigrams": [], "words": [], 
        "meta": {"is_std": false, "id": "en_small", "name": "En Small", "version": "1.0", "author": "", "description": ""}
    }"#).await?;

    // Request
    let mut req = JobRequest::default();
    req.config.definition.meta.name = "default".to_string(); // Match created asset
    req.config.cost_matrix = CostMatrixSourceDto::Predefined {
        id: "matrix.json".to_string(),
        hash: None,
    };

    // Run 1
    let (id1, _) = OptimizationUseCase::prepare_session(&provider, &req).await?;

    // Update Cost Matrix content
    fs::write(
        temp.path().join("system/weights/matrix.json"),
        r#"{
        "meta": {"version": "1.1", "description": "test-modified", "unit": "pts"},
        "models": {
            "model_a_row_staggered": {
                "description": "test",
                "static_costs": {
                    "universal_hand": {
                        "fingers": {
                            "index": {"base": {"r0": 100, "r1": 100, "r2": 100, "r3": 100}},
                            "middle": {"base": {"r0": 100, "r1": 100, "r2": 100, "r3": 100}},
                            "ring": {"base": {"r0": 100, "r1": 100, "r2": 100, "r3": 100}},
                            "pinky": {"base": {"r0": 100, "r1": 100, "r2": 100, "r3": 100}},
                            "thumb": {"pos_1": 100, "pos_2": 100, "pos_3": 100}
                        }
                    }
                }
            }
        },
        "dynamic_rules": {"sequence_modifiers": {}, "penalties": {}, "constraints": {}}
    }"#,
    )
    .await?;

    // Run 2
    let (id2, _) = OptimizationUseCase::prepare_session(&provider, &req).await?;

    assert_ne!(
        id1.hash, id2.hash,
        "Job ID should change when Cost Matrix content changes"
    );

    Ok(())
}
