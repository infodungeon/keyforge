// libs/keyforge-model/src/types/mod.rs

/// Biomechanical types (Hands, Fingers).
pub mod biomechanical;
/// Deterministic rational types.
pub mod fraction;
/// Keyboard geometry types (Rows, Columns).
pub mod geometry;
/// Key identifiers (Indices, Codes).
pub mod indices;
/// Deterministic math types and traits.
pub mod math;
/// Domain-specific newtypes.
pub mod newtypes;
/// Path types (`SafePath`).
pub mod path;
/// Optimization and analysis result types.
pub mod results;
/// Scoring types (Score, Weight).
pub mod scoring;

pub use biomechanical::{Finger, FingerIndex, Hand, HandIndex, SpaceHandPreference};
pub use fraction::Fraction;
pub use geometry::{ColIndex, RowIndex};
pub use indices::{KeyCode, KeyIndex};
pub use math::{FixedPointMath, Movement, Point, SpatialUnit, TrigramFlow};
pub use newtypes::{
    DurationMs, IterationCount, LatencyMs, PatienceCount, ReheatCount, ScalingFactor, Seed,
    Temperature,
};
pub use path::SafePath;
pub use results::{
    AnalysisReport, MetricViolation, OptimizationResult, ScoringResult, SwapSuggestion,
};
pub use scoring::{FixedWeight, Score, Weight};

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::constants::SCORE_SCALE;

    #[test]
    fn test_score_overflow_saturation() -> anyhow::Result<()> {
        let max = Score::MAX;
        // Saturating add
        assert_eq!(max + Score::from_scaled_i64(1), Score::MAX);
        // Saturating sub
        assert_eq!(Score::MIN - Score::from_scaled_i64(1), Score::MIN);
        // Saturating mul
        assert_eq!(max * 2, Score::MAX);
        Ok(())
    }

    #[test]
    fn test_score_checked_ops() -> anyhow::Result<()> {
        let max = Score::MAX;
        assert!(max.checked_add(Score::from_scaled_i64(1)).is_none());
        assert!(Score::MIN.checked_sub(Score::from_scaled_i64(1)).is_none());
        Ok(())
    }

    #[test]
    fn test_score_scaling() -> anyhow::Result<()> {
        let s = Score::from_f32(1.0).map_err(|e| anyhow::anyhow!(e))?;
        assert_eq!(s.to_f32(), 1.0);
        assert_eq!(s.raw(), SCORE_SCALE);
        Ok(())
    }

    #[test]
    fn test_hand_index_try_from() -> anyhow::Result<()> {
        assert!(HandIndex::try_from(0).is_ok());
        assert!(HandIndex::try_from(1).is_ok());
        assert!(HandIndex::try_from(2).is_err());
        Ok(())
    }

    #[test]
    fn test_basic_types_coverage() -> anyhow::Result<()> {
        // KeyIndex
        let ki = KeyIndex::new(10);
        assert_eq!(format!("{ki}"), "10");
        assert_eq!(usize::from(ki), 10);
        assert_eq!(KeyIndex::from(10usize), ki);

        // KeyCode
        let kc = KeyCode::new(97);
        assert_eq!(format!("{kc}"), "97");
        assert_eq!(u16::from(kc), 97);
        assert_eq!(KeyCode::from(97u16), kc);

        // HandIndex
        let hi = HandIndex::LEFT;
        assert_eq!(hi.raw(), 0);
        assert_eq!(hi.as_usize(), 0);
        assert!(hi.is_left());
        assert!(!hi.is_right());
        assert!(HandIndex::RIGHT.is_right());
        assert_eq!(HandIndex::default(), HandIndex::LEFT);

        // FingerIndex
        let fi = FingerIndex::INDEX;
        assert_eq!(fi.raw(), 1);
        assert_eq!(fi.as_usize(), 1);
        assert_eq!(fi.distance(FingerIndex::PINKY), 3);
        assert_eq!(fi.diff(FingerIndex::PINKY), -3);
        assert!(!fi.is_weak());
        assert!(FingerIndex::RING.is_weak());
        assert_eq!(FingerIndex::default(), FingerIndex::INDEX);
        assert!(FingerIndex::try_from(1).is_ok());
        assert!(FingerIndex::try_from(5).is_err());

        // Row/Col Index
        assert_eq!(RowIndex::new(5) - RowIndex::new(2), 3);
        assert_eq!(ColIndex::new(5) - ColIndex::new(2), 3);
        assert_eq!(RowIndex::default().raw(), 0);
        assert_eq!(ColIndex::default().raw(), 0);

        // SpaceHandPreference
        assert_eq!(
            SpaceHandPreference::default(),
            SpaceHandPreference::Bilateral
        );
        Ok(())
    }

    #[test]
    fn test_score_extended() -> anyhow::Result<()> {
        assert_eq!(Score::ZERO.raw(), 0);
        assert_eq!(Score::MAX.raw(), i64::MAX);
        assert_eq!(Score::MIN.raw(), i64::MIN);

        let s = Score::from_scaled_i64(100);
        assert_eq!(s.raw(), 100);

        // Score::from_f32 errors
        assert!(Score::from_f32(f32::NAN).is_err());
        assert!(Score::from_f32(1e20).is_err()); // Overflow

        // Score Ops
        let s1 = Score::from_scaled_i64(100);
        let s2 = Score::from_scaled_i64(50);
        assert_eq!((s1 + s2).raw(), 150);
        assert_eq!((s1 - s2).raw(), 50);
        assert_eq!((-s1).raw(), -100);
        assert_eq!((s1 * 2).raw(), 200);
        Ok(())
    }
}

#[keyforge_testing_macros::kf_test]
mod fuzz {
    use super::*;
    use proptest::prelude::*;

    fn score_strategy() -> impl Strategy<Value = i64> {
        any::<i64>()
    }

    proptest! {
        #[test]
        fn fuzz_score_ops(a in score_strategy(), b in score_strategy()) {
            let s1 = Score::from_scaled_i64(a);
            let s2 = Score::from_scaled_i64(b);
            let _ = s1 + s2;
            let _ = s1 - s2;
            let _ = -s1;
        }
    }
}
