use keyforge_infra::net::client::{ClientConfig, HiveClient};
use keyforge_protocol::{JobRequest, ResultSubmission, NodeRequest, JobResponse, PROTOCOL_VERSION};
use keyforge_security as crypto;
use keyforge_physics::{EngineCompilationContext, EngineFactory};
use keyforge_model::{Corpus, Keyboard, Rubric, KeycodeRegistry, Layout};
use keyforge_model::types::KeyCode;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinSet;

const HIVE_URL: &str = "http://localhost:3000";
const HIVE_SECRET: &str = "test_secret";
const CONCURRENT_WORKERS: usize = 10; // Lowered further for compilation speed
const RESULTS_PER_WORKER: usize = 10;

#[tokio::test]
#[ignore = "Requires running Hive instance"]
async fn test_load_resilience() -> anyhow::Result<()> {
    // 1. Setup Client
    let config = ClientConfig {
        api_url: HIVE_URL.to_string(),
        asset_url: HIVE_URL.replace("3000", "3001"),
        secret: Some(HIVE_SECRET.to_string()),
        ..Default::default()
    };
    let client = Arc::new(HiveClient::new(config)?);

    // 2. Setup Local Engine for Truth Scoring
    // We'll use a minimal setup to match what Hive produces for a default JobRequest
    let registry = Arc::new(KeycodeRegistry::new_with_alphas());
    let kb_def = keyforge_model::geometry::KeyboardDefinition::default(); // Should match Hive default if we don't change name
    // Actually, I changed name to "corne" in previous turn.
    // Let's use standard assets from data/system
    
    let kb_data = std::fs::read("../../data/system/keyboards/models/corne.mpk.zst")?;
    let kb_def: keyforge_model::geometry::KeyboardDefinition = rmp_serde::from_slice(&zstd::decode_all(&kb_data[..])?)?;
    let kb = Keyboard::new(kb_def.geometry.keys, kb_def.geometry.home_row, "corne".into())?;
    let kb = Arc::new(kb);
    
    let corpus = Arc::new(Corpus::default()); // en_small empty by default but matches Hive if not seeded
    let rubric = Arc::new(Rubric::default());
    let cost_model = Arc::new(keyforge_model::testing::mock_cost_model());
    
    let ctx = EngineCompilationContext {
        keyboard: kb.clone(),
        corpus: corpus.clone(),
        rubric: rubric.clone(),
        cost_model: cost_model.clone(),
        engine_config: keyforge_model::config::EngineConfig::default(),
    };
    let engine = Arc::new(EngineFactory::new_scalar(&ctx)?);
    println!("⚙️ Local Verification Engine Compiled ({} keys)", engine.key_count());

    // 3. Register Job
    let mut job_req = JobRequest::default();
    job_req.config.definition.meta.name = "corne".into();
    
    let resp = client.post("jobs").json(&job_req).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await?;
        anyhow::bail!("Job registration failed ({}): {}", status, err_body);
    }
    let job_resp: JobResponse = resp.json().await?;
    let job_id = job_resp.job_id;
    println!("📝 Registered Job: {}", job_id);

    // 4. Flood with Results
    let start = std::time::Instant::now();
    let mut set = JoinSet::new();

    for i in 0..CONCURRENT_WORKERS {
        let c = client.clone();
        let jid = job_id.clone();
        let eng = engine.clone();
        let reg = registry.clone();
        let run_id = fastrand::u32(..);
        
        set.spawn(async move {
            let node_id = format!("load-worker-{}-{:x}", i, run_id);
            let (sk, pk) = crypto::generate_keypair();
            
            // Register Node
            let reg_req = NodeRequest {
                version: PROTOCOL_VERSION,
                node_id: node_id.clone(),
                hostname: "load-test-host".into(),
                cpu_cores: 1,
                cpu_model: "LoadTest".into(),
                capabilities: Vec::new(),
                cores: 1,
                l2_cache_kb: Some(0),
                ops_per_sec: 1000.0,
                public_key: Some(pk),
            };
            
            let reg_resp = c.post("nodes/register").json(&reg_req).send().await.unwrap();
            if !reg_resp.status().is_success() {
                let status = reg_resp.status();
                let txt = reg_resp.text().await.unwrap();
                panic!("Node registration failed ({}): {}", status, txt);
            }
            
            // Generate valid layout for this keyboard
            let code_a = reg.resolve_token("A").unwrap();
            let key_codes: Vec<KeyCode> = vec![code_a; eng.key_count()];
            let layout_struct = Layout::new_unchecked(key_codes);
            let layout_str = vec!["A"; eng.key_count()].join(" ");
            
            // Calculate real score
            let score_obj = eng.score(&layout_struct).unwrap();
            let raw_score = score_obj.raw();
            let score_f32 = score_obj.to_f32();

            for k in 0..RESULTS_PER_WORKER {
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                let nonce = (i * 10000 + k) as u64;
                
                let sig = crypto::sign_result_fixed(&sk, &jid, &layout_str, raw_score, timestamp, nonce).unwrap();
                
                let sub = ResultSubmission {
                    version: PROTOCOL_VERSION,
                    job_id: jid.clone(),
                    layout: layout_str.clone(),
                    score: score_f32,
                    raw_score,
                    timestamp,
                    nonce,
                    node_id: node_id.clone(),
                    signature: sig,
                };
                
                let resp = c.post("results").json(&sub).send().await.unwrap();
                if !resp.status().is_success() {
                    let status = resp.status();
                    let txt = resp.text().await.unwrap();
                    panic!("Submission failed ({}): {}", status, txt);
                }
            }
        });
    }

    while let Some(res) = set.join_next().await {
        res?;
    }

    let elapsed = start.elapsed();
    let total_reqs = CONCURRENT_WORKERS * RESULTS_PER_WORKER;
    println!("✅ Load Test Complete. {} signed reqs in {:.2?}", total_reqs, elapsed);
    
    // 5. Verify results reach Hive
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    let status_url = format!("jobs/{}/status", job_id);
    let status_resp: keyforge_protocol::JobDetailedStatus = client.get(&status_url).send().await?.json().await?;
    
    println!("📊 Hive Final Status: Total Samples = {}", status_resp.total_samples);
    assert!(status_resp.total_samples > 0, "No samples reached the database!");
    
    Ok(())
}
