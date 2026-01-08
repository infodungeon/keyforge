// libs/keyforge-protocol/tests/serialization_security.rs

//! Integration tests for protocol serialization security. Verifies the protocol's
//! defensive limits against malformed or oversized payloads, ensuring that deserialization
//! of `JobRequest` correctly rejects excessive biometric samples to prevent resource
//! exhaustion attacks.


use keyforge_protocol::JobRequest;
use keyforge_model::constants::ASSET_COST_MATRIX;

#[test]
fn test_deserialize_dos_protection_biometrics() {
    // Create a JSON payload with 100,001 biometric samples
    // Note: "name" is required in meta
    let mut json = String::from(format!(r#"
    {{
        "version": 1,
        "definition": {{ 
            "meta": {{ "name": "SecurityTest" }}, 
            "geometry": {{ "keys": [], "prime_slots": [], "med_slots": [], "low_slots": [], "home_row": 1 }} 
        }},
        "weights": {{}},
        "params": {{ "search_epochs": 1, "search_steps": 1, "search_patience": 1, "search_patience_threshold": 0.1, "temp_min": 0.1, "temp_max": 1.0, "opt_limit_fast": 1, "opt_limit_slow": 1, "reheats": 0, "reheat_factor": 0.5 }},
        "pinned_keys": [],
        "corpora": [],
        "cost_matrix": {{ "type": "Predefined", "data": "{}" }},
        "biometrics": ["#, ASSET_COST_MATRIX));

    for i in 0..100_001 {
        if i > 0 { json.push(','); }
        json.push_str(r#"{ "bigram": "ab", "ms": 10.0, "timestamp": 0 }"#);
    }
    json.push_str("]}");

    let res: Result<JobRequest, _> = serde_json::from_str(&json);
    
    assert!(res.is_err(), "Should reject > 100k items");
    let err = res.unwrap_err().to_string();
    // We expect the error to be about the vector limit, not a missing field
    assert!(err.contains("exceeds limit"), "Unexpected error: {}", err);
}
