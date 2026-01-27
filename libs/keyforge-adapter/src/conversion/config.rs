use keyforge_model::config;

/// Converts a protocol-level corpus source into a domain-level source.
#[must_use]
pub fn to_domain_corpus_source(s: &config::CorpusSource) -> config::CorpusSource {
    s.clone()
}

/// Converts protocol-level scoring weights into a domain-level evaluation rubric.
#[must_use]
pub fn to_domain_rubric(w: &config::ScoringWeights) -> keyforge_model::Rubric {
    keyforge_model::Rubric {
        finger_effort: w.get_finger_penalty_scale(),
        travel_lat: w.get_weight_lateral_travel().to_f32(),
        travel_vert: w.get_weight_vertical_travel().to_f32(),
        sfb_base: w.get_penalty_sfb_base().to_f32(),
        sfb_lateral: w.get_penalty_sfb_lateral().to_f32(),
        sfb_lateral_weak: w.get_penalty_sfb_lateral_weak().to_f32(),
        sfb_diagonal: w.get_penalty_sfb_diagonal().to_f32(),
        sfb_long: w.get_penalty_sfb_long().to_f32(),
        threshold_sfb_long_row_diff: w.get_threshold_sfb_long_row_diff(),
        penalty_scissor: w.get_penalty_scissor().to_f32(),
        threshold_scissor_row_diff: w.get_threshold_scissor_row_diff(),
        redirect: w.get_penalty_redirect().to_f32(),
        roll_bonus: w.get_bonus_bigram_roll_in().to_f32(),
        roll_out_bonus: w.get_bonus_bigram_roll_out().to_f32(),
        trigram_coverage: w.get_trigram_coverage().to_f32(),
        trigram_limit: w.get_loader_trigram_limit(),
    }
}

/// Converts protocol-level search parameters into domain-level search configuration.
#[must_use]
pub fn to_domain_config(p: &config::SearchParams, seed: u64) -> keyforge_model::SearchConfig {
    keyforge_model::SearchConfig::Annealing {
        steps: p.get_search_steps(),
        start_temp: p.get_temp_max(),
        end_temp: p.get_temp_min(),
        seed,
        patience: p.get_search_patience(),
        reheats: p.get_reheats(),
        reheat_factor: p.get_reheat_factor(),
        include_thumbs: p.include_thumbs,
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use keyforge_model::config::{ScoringWeights, SearchParams};

    #[test]
    fn test_to_domain_rubric_conversion() {
        let mut proto_weights = ScoringWeights::default();
        proto_weights
            .weights
            .insert("penalty_sfb_base".to_string(), 100.0);

        let domain_rubric = to_domain_rubric(&proto_weights);
        assert!((domain_rubric.sfb_base - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_to_domain_config_conversion() {
        let mut proto_params = SearchParams::default();
        proto_params
            .params
            .insert("search_steps".to_string(), 100_000.0);

        let domain_config = to_domain_config(&proto_params, 42);
        match domain_config {
            keyforge_model::SearchConfig::Annealing { steps, seed, .. } => {
                assert_eq!(steps, 100_000);
                assert_eq!(seed, 42);
            }
        }
    }

    #[test]
    fn test_to_domain_corpus_source() {
        let src = config::CorpusSource {
            id: "en".into(),
            weight: 1.0,
            hash: Some("h".into()),
        };
        let res = to_domain_corpus_source(&src);
        assert_eq!(res.id, "en");
        assert_eq!(res.weight, 1.0);
        assert_eq!(res.hash, Some("h".into()));
    }
}
