// libs/keyforge-model/src/testing.rs
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::field_reassign_with_default
    )
)]

use crate::types::SpatialUnit;
use crate::{
    ColIndex, Corpus, CostModel, FingerIndex, HandIndex, KeyCode, KeyNode, Keyboard, RowIndex,
    Rubric,
};
use proptest::arbitrary::Arbitrary;
use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;
use std::sync::Arc;

impl Arbitrary for KeyCode {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0u16..500).prop_map(KeyCode::new).boxed()
    }
}

impl Arbitrary for HandIndex {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0u8..=1).prop_map(HandIndex::new).boxed()
    }
}

impl Arbitrary for FingerIndex {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0u8..=4).prop_map(FingerIndex::new_unchecked).boxed()
    }
}

impl Arbitrary for RowIndex {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (any::<i8>()).prop_map(RowIndex::new).boxed()
    }
}

impl Arbitrary for ColIndex {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (any::<i8>()).prop_map(ColIndex::new).boxed()
    }
}

impl Arbitrary for KeyNode {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<usize>(),
            ".*",           // label
            -15.0f32..15.0, // x
            -15.0f32..15.0, // y
            any::<HandIndex>(),
            any::<FingerIndex>(),
            any::<RowIndex>(),
            any::<ColIndex>(),
            any::<bool>(),
        )
            .prop_map(
                |(index, label, x, y, hand, finger, row, col, is_home)| Self {
                    index,
                    label,
                    x: SpatialUnit::from_f32(x),
                    y: SpatialUnit::from_f32(y),
                    w: 1.0,
                    h: 1.0,
                    r: 0.0,
                    rx: SpatialUnit::default(),
                    ry: SpatialUnit::default(),
                    hand,
                    finger,
                    row,
                    col,
                    is_home,
                    is_stretch: false,
                },
            )
            .boxed()
    }
}

impl Arbitrary for Keyboard {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // Generate a small but valid keyboard
        prop::collection::vec(any::<KeyNode>(), 10..60)
            .prop_map(|keys| {
                // Ensure unique indices for keys
                let mut keys = keys;
                for (i, key) in keys.iter_mut().enumerate() {
                    key.index = i;
                }
                #[allow(clippy::unwrap_used)]
                Keyboard::new(keys, RowIndex::new(1), "test".into()).unwrap()
            })
            .boxed()
    }
}

impl Arbitrary for Corpus {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            prop::collection::vec(0u64..100, 256),
            prop::collection::vec((0u16..255, 0u16..255, 0u32..50), 0..20),
            prop::collection::vec((0u16..255, 0u16..255, 0u16..255, 0u32..20), 0..10),
        )
            .prop_map(|(char_freqs, mut bigrams, mut trigrams)| {
                let mut char_freqs_full = vec![0u64; 65536];
                for (i, &f) in char_freqs.iter().enumerate() {
                    char_freqs_full[i] = f;
                }
                // Sorting required by Corpus::validate/merge but not strictly for existence
                bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
                trigrams.sort_unstable_by(|a, b| {
                    a.0.cmp(&b.0)
                        .then(a.1.cmp(&b.1))
                        .then(a.2.cmp(&b.2))
                });

                Corpus {
                    meta: crate::corpus::CorpusMetadata::default(),
                    char_freqs: Arc::from(char_freqs_full),
                    bigrams: Arc::from(bigrams),
                    trigrams: Arc::from(trigrams),
                    words: Arc::from(vec![]),
                }
            })
            .boxed()
    }
}

impl Arbitrary for Rubric {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            prop::collection::vec(-500.0f32..500.0, 5),
            -500.0f32..500.0,   // travel_lat
            -500.0f32..500.0,   // travel_vert
            -500.0f32..500.0, // sfb_base
            -500.0f32..500.0,  // sfb_lateral
            -500.0f32..500.0,  // sfb_lateral_weak
            -500.0f32..500.0,  // sfb_diagonal
            -500.0f32..500.0,  // sfb_long
            -500.0f32..500.0,  // penalty_scissor
            -500.0f32..500.0,  // redirect
            -500.0f32..500.0,  // roll_bonus
        )
            .prop_map(
                |(effort, tlat, tvert, sfb, slat, slweak, sdiag, slong, pscis, redir, roll)| {
                    Rubric::builder()
                        .finger_effort(effort.try_into().unwrap_or([0.0; 5]))
                        .travel_lat(tlat)
                        .travel_vert(tvert)
                        .sfb_base(sfb)
                        .sfb_lateral(slat)
                        .sfb_lateral_weak(slweak)
                        .sfb_diagonal(sdiag)
                        .sfb_long(slong)
                        .penalty_scissor(pscis)
                        .redirect(redir)
                        .roll_bonus(roll)
                        .build()
                },
            )
            .boxed()
    }
}

/// Creates a minimal, valid cost model for testing.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn mock_cost_model() -> CostModel {
    let mut base_zone = crate::cost_model::RowCosts::new();
    for r in -128..=127 {
        base_zone.insert(RowIndex::new(r as i8), crate::types::Score::ZERO);
    }

    let index_zones = crate::cost_model::FingerReach {
        base: base_zone,
        inner: std::collections::HashMap::new(),
        outer: std::collections::HashMap::new(),
    };

    let mut fingers = std::collections::HashMap::new();
    fingers.insert(
        "thumb".into(),
        crate::cost_model::FingerDefinition::Thumb(std::collections::HashMap::new()),
    );
    fingers.insert(
        "index".into(),
        crate::cost_model::FingerDefinition::Standard(index_zones.clone()),
    );
    fingers.insert(
        "middle".into(),
        crate::cost_model::FingerDefinition::Standard(index_zones.clone()),
    );
    fingers.insert(
        "ring".into(),
        crate::cost_model::FingerDefinition::Standard(index_zones.clone()),
    );
    fingers.insert(
        "pinky".into(),
        crate::cost_model::FingerDefinition::Standard(index_zones),
    );

    let mut static_costs = std::collections::HashMap::new();
    static_costs.insert(
        "universal_hand".into(),
        crate::cost_model::HandDefinition { fingers },
    );

    let mut models = std::collections::HashMap::new();
    models.insert(
        "model_a_row_staggered".into(),
        crate::cost_model::ModelDefinition {
            description: "test".into(),
            static_costs,
        },
    );

    CostModel {
        meta: crate::cost_model::CostModelMeta {
            version: "2.0".into(),
            description: "test".into(),
            unit: "pts".into(),
        },
        models,
        dynamic_rules: crate::cost_model::DynamicRules::default(),
    }
}

/// Sets up a minimal environment with Keyboard, Corpus, Rubric, and `CostModel`.
#[must_use]
#[allow(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
pub fn setup_minimal_assets() -> (Keyboard, Corpus, Rubric, CostModel) {
    let keys: Vec<KeyNode> = (0..3)
        .map(|i| KeyNode {
            index: i,
            label: format!("k{i}"),
            hand: HandIndex::new(0),
            finger: FingerIndex::new(i as u8),
            x: SpatialUnit::from_f32(i as f32),
            y: SpatialUnit::default(),
            ..Default::default()
        })
        .collect();
    let kb = Keyboard::new(keys, RowIndex::new(0), "test".into()).unwrap();

    let mut corpus = Corpus::default();
    let mut freqs = corpus.char_freqs.to_vec();
    freqs[97] = 100;
    freqs[98] = 200;
    corpus.char_freqs = Arc::from(freqs);
    corpus.bigrams = Arc::from(vec![(97, 98, 50)]);

    let cm = mock_cost_model();
    (kb, corpus, Rubric::default(), cm)
}
