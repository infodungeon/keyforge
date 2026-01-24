#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-protocol/tests/contract_integration.rs

    use keyforge_model::{types::HandIndex, KeyNode, Validator};
    use keyforge_protocol::{JobRequest, PROTOCOL_VERSION};

    #[test]
    fn test_full_job_request_lifecycle() {
        let mut req = JobRequest::default();
        req.version = PROTOCOL_VERSION;

        // 1. Initial default should be invalid (empty geometry)
        assert!(req.validate().is_err());

        // 2. Setup minimum valid domain data
        req.config.definition.geometry.keys.push(KeyNode {
            hand: HandIndex(0),
            w: 1.0,
            h: 1.0,
            ..Default::default()
        });
        req.config.definition.geometry.home_row = 0;
        req.config
            .definition
            .geometry
            .prime_slots
            .push(keyforge_model::KeyIndex(0));

        // 3. Should now be valid
        assert!(req.validate().is_ok());

        // 4. Round-trip JSON
        let json = serde_json::to_string(&req).unwrap();
        let recovered: JobRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.version, PROTOCOL_VERSION);
        assert_eq!(recovered.config.definition.geometry.keys.len(), 1);
    }
}
