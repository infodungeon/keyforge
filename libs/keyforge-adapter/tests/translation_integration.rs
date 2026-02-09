#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    // libs/keyforge-adapter/tests/translation_integration.rs

    use keyforge_adapter::conversion;
    use keyforge_model::config::ScoringWeights;

    #[test]
    fn test_end_to_end_rubric_translation() {
        let mut weights = ScoringWeights::default();
        weights
            .weights
            .insert("penalty_sfb_base".to_string(), 1234.5);

        let rubric = conversion::to_domain_rubric(&weights);
        assert_eq!(rubric.sfb_base(), keyforge_model::types::FixedWeight::from_f32(1234.5).unwrap());
    }
}
