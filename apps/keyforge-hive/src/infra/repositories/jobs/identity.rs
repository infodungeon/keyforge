use keyforge_protocol::JobRequest;
use sha2::{Digest, Sha256};
use crate::constants::JOB_IDENTITY_PRECISION;

pub fn calculate_job_identity(req: &JobRequest) -> Result<String, String> {
    fn norm(v: f32) -> f32 {
        if v == 0.0 { 0.0 } else { (v * JOB_IDENTITY_PRECISION).round() / JOB_IDENTITY_PRECISION }
    }

    let kb_meta = &req.config.definition.meta;
    let lock_key = format!("{}{}{}", kb_meta.name, kb_meta.author, kb_meta.version);
    
    // Unique Hash (for Lock/Geometry)
    let mut hasher = Sha256::new();
    hasher.update(lock_key.as_bytes());
    let unique_hash = hex::encode(hasher.finalize());

    let mut w = req.config.weights.clone();
    let sfb_base = w.get_penalty_sfb_base();
    w.weights.insert("penalty_sfb_base".to_string(), norm(sfb_base));
    
    let w_str = serde_json::to_string(&w).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(w_str.as_bytes());
    let w_hash = hex::encode(hasher.finalize());

    // Search Params
    let mut p = req.config.params.clone();
    let temp_min = p.get_temp_min();
    p.params.insert("temp_min".to_string(), norm(temp_min));
    
    let p_json = serde_json::to_string(&p).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(p_json.as_bytes());
    let p_hash = hex::encode(hasher.finalize());

    // Final Job Identity Hash
    let mut hasher = Sha256::new();
    hasher.update(unique_hash.as_bytes());
    hasher.update(w_hash.as_bytes());
    hasher.update(p_hash.as_bytes());
    
    Ok(hex::encode(hasher.finalize()))
}