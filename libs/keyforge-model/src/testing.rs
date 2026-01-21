// libs/keyforge-model/src/testing.rs
#![allow(clippy::unwrap_used)]

use crate::{KeyCode, HandIndex, FingerIndex, RowIndex, ColIndex, KeyNode, Keyboard, Corpus, Rubric};
use proptest::prelude::*;
use proptest::arbitrary::Arbitrary;
use proptest::strategy::BoxedStrategy;

impl Arbitrary for KeyCode {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0u16..500).prop_map(KeyCode).boxed()
    }
}

impl Arbitrary for HandIndex {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0u8..=1).prop_map(HandIndex).boxed()
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
        (any::<i8>()).prop_map(RowIndex).boxed()
    }
}

impl Arbitrary for ColIndex {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (any::<i8>()).prop_map(ColIndex).boxed()
    }
}

impl Arbitrary for KeyNode {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<usize>(),
            ".*", // label
            -15.0f32..15.0, // x
            -15.0f32..15.0, // y
            any::<HandIndex>(),
            any::<FingerIndex>(),
            any::<RowIndex>(),
            any::<ColIndex>(),
            any::<bool>(),
        )
            .prop_map(|(index, label, x, y, hand, finger, row, col, is_home)| Self {
                index,
                label,
                x,
                y,
                w: 1.0,
                h: 1.0,
                r: 0.0,
                rx: 0.0,
                ry: 0.0,
                hand,
                finger,
                row,
                col,
                is_home,
                is_stretch: false,
            })
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
                Keyboard::new(keys, 1, "test".into()).unwrap()
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
            .prop_map(|(char_freqs, bigrams, trigrams)| {
                let mut char_freqs_full = vec![0u64; 65536];
                for (i, &f) in char_freqs.iter().enumerate() {
                    char_freqs_full[i] = f;
                }
                let mut c = Corpus {
                    meta: crate::corpus::CorpusMetadata::default(),
                    char_freqs: char_freqs_full,
                    bigrams,
                    trigrams,
                    words: Vec::new(),
                };
                // Sorting required by Corpus::validate/merge but not strictly for existence
                c.bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
                c.trigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
                c
            })
            .boxed()
    }
}

impl Arbitrary for Rubric {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            prop::collection::vec(0.0f32..10.0, 5),
            0.0f32..20.0,   // travel_lat
            0.0f32..20.0,   // travel_vert
            0.0f32..1000.0, // sfb_base
            0.0f32..500.0,  // sfb_lateral
            0.0f32..500.0,  // sfb_lateral_weak
            0.0f32..500.0,  // sfb_diagonal
            0.0f32..500.0,  // sfb_long
            0.0f32..500.0,  // penalty_scissor
            0.0f32..500.0,  // redirect
            0.0f32..500.0,  // roll_bonus
        )
            .prop_map(|(effort, tlat, tvert, sfb, slat, slweak, sdiag, slong, pscis, redir, roll)| {
                let mut r = Rubric::default();
                for i in 0..5 {
                    r.finger_effort[i] = effort[i];
                }
                r.travel_lat = tlat;
                r.travel_vert = tvert;
                r.sfb_base = sfb;
                r.sfb_lateral = slat;
                r.sfb_lateral_weak = slweak;
                r.sfb_diagonal = sdiag;
                r.sfb_long = slong;
                r.penalty_scissor = pscis;
                r.redirect = redir;
                r.roll_bonus = roll;
                r
            })
            .boxed()
    }
}
