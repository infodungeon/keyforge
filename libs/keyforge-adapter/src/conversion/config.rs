use keyforge_model::config;

/// Converts a protocol-level corpus source into a domain-level source.
#[must_use] 
pub fn to_domain_corpus_source(s: &config::CorpusSource) -> config::CorpusSource {
    config::CorpusSource {
        id: s.id.clone(),
        weight: s.weight,
        hash: s.hash.clone(),
    }
}

/// Converts protocol-level scoring weights into a domain-level evaluation rubric.
#[must_use] 
pub fn to_domain_rubric(w: &config::ScoringWeights) -> keyforge_model::Rubric {
    keyforge_model::Rubric {
        finger_effort: w.get_finger_penalty_scale(),
        travel_lat: w.get_weight_lateral_travel(),
        travel_vert: w.get_weight_vertical_travel(),
        sfb_base: w.get_penalty_sfb_base(),
        sfb_lateral: w.get_penalty_sfb_lateral(),
        sfb_lateral_weak: w.get_penalty_sfb_lateral_weak(),
        sfb_diagonal: w.get_penalty_sfb_diagonal(),
        sfb_long: w.get_penalty_sfb_long(),
        threshold_sfb_long_row_diff: w.get_threshold_sfb_long_row_diff(),
        penalty_scissor: w.get_penalty_scissor(),
        threshold_scissor_row_diff: w.get_threshold_scissor_row_diff(),
        redirect: w.get_penalty_redirect(),
        roll_bonus: w.get_bonus_bigram_roll_in(),
        trigram_coverage: w.get_trigram_coverage(),
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
