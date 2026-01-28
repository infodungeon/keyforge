// libs/keyforge-model/src/types/mod.rs

/// Biomechanical types (Hands, Fingers).
pub mod biomechanical;
/// Keyboard geometry types (Rows, Columns).
pub mod geometry;
/// Key identifiers (Indices, Codes).
pub mod indices;
/// Domain-specific newtypes.
pub mod newtypes;
/// Optimization and analysis result types.
pub mod results;
/// Scoring types (Score, Weight).
pub mod scoring;

pub use biomechanical::{FingerIndex, HandIndex, SpaceHandPreference};
pub use geometry::{ColIndex, RowIndex};
pub use indices::{KeyCode, KeyIndex};
pub use newtypes::{
    CorpusId, DurationMs, IterationCount, KeyboardId, LatencyMs, LayoutId, PatienceCount,
    Percentage, Ratio, ReheatCount, ScalingFactor, Seed, Temperature, UserId,
};
pub use results::{
    AnalysisReport, Heatmaps, MetricBreakdown, MetricViolation, OptimizationResult, ScoreSummary,
    ScoringResult, SwapSuggestion, TravelStatistics,
};
pub use scoring::{Score, Weight};

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use crate::constants::SCORE_SCALE;

    #[test]
    fn test_score_overflow_saturation() {
        let max = Score::MAX;
        // Saturating add
        assert_eq!(max + Score(1), Score::MAX);
        // Saturating sub
        assert_eq!(Score::MIN - Score(1), Score::MIN);
        // Saturating mul
        assert_eq!(max * 2, Score::MAX);
    }

    #[test]
    fn test_score_checked_ops() {
        let max = Score::MAX;
        assert!(max.checked_add(Score(1)).is_none());
        assert!(Score::MIN.checked_sub(Score(1)).is_none());
    }

    #[test]
    fn test_score_scaling() {
        let s = Score::from_f32(1.0).unwrap();
        assert_eq!(s.to_f32(), 1.0);
        assert_eq!(s.0, SCORE_SCALE as i64);
    }

    #[test]
    fn test_hand_index_try_from() {
        assert!(HandIndex::try_from(0).is_ok());
        assert!(HandIndex::try_from(1).is_ok());
        assert!(HandIndex::try_from(2).is_err());
    }

    #[test]
    fn test_basic_types_coverage() {
        // KeyIndex
        let ki = KeyIndex(10);
        assert_eq!(format!("{ki}"), "10");
        assert_eq!(usize::from(ki), 10);
        assert_eq!(KeyIndex::from(10usize), ki);

        // KeyCode
        let kc = KeyCode(97);
        assert_eq!(format!("{kc}"), "97");
        assert_eq!(u16::from(kc), 97);
        assert_eq!(KeyCode::from(97u16), kc);

        // HandIndex
        let hi = HandIndex::LEFT;
        assert_eq!(hi.as_u8(), 0);
        assert_eq!(hi.as_usize(), 0);
        assert!(hi.is_left());
        assert!(!hi.is_right());
        assert!(HandIndex::RIGHT.is_right());
        assert_eq!(HandIndex::default(), HandIndex::LEFT);

        // FingerIndex
        let fi = FingerIndex::INDEX;
        assert_eq!(fi.as_u8(), 1);
        assert_eq!(fi.as_usize(), 1);
        assert_eq!(fi.distance(FingerIndex::PINKY), 3);
        assert_eq!(fi.diff(FingerIndex::PINKY), -3);
        assert!(!fi.is_weak());
        assert!(FingerIndex::RING.is_weak());
        assert_eq!(FingerIndex::default(), FingerIndex::INDEX);
        assert!(FingerIndex::try_from(1).is_ok());
        assert!(FingerIndex::try_from(5).is_err());

        // Row/Col Index
        assert_eq!(RowIndex(5) - RowIndex(2), 3);
        assert_eq!(ColIndex(5) - ColIndex(2), 3);
        assert_eq!(RowIndex::default().0, 0);
        assert_eq!(ColIndex::default().0, 0);

        // SpaceHandPreference
        assert_eq!(
            SpaceHandPreference::default(),
            SpaceHandPreference::Bilateral
        );

        // Percentage
        assert!(Percentage::try_from(50.0).is_ok());
        assert!(Percentage::try_from(101.0).is_err());
        assert!(Percentage::try_from(-1.0).is_err());

        // Ratio
        assert!(Ratio::try_from(0.5).is_ok());
        assert!(Ratio::try_from(1.1).is_err());
        assert!(Ratio::try_from(-0.1).is_err());
    }

    #[test]
    fn test_score_extended() {
        assert_eq!(Score::ZERO.0, 0);
        assert_eq!(Score::MAX.0, i64::MAX);
        assert_eq!(Score::MIN.0, i64::MIN);

        let s = Score::from_scaled_i64(100);
        assert_eq!(s.0, 100);

        // Score::from_f32 errors
        assert!(Score::from_f32(f32::NAN).is_err());
        assert!(Score::from_f32(1e20).is_err()); // Overflow

        // Score Ops
        let s1 = Score(100);
        let s2 = Score(50);
        assert_eq!((s1 + s2).0, 150);
        assert_eq!((s1 - s2).0, 50);
        assert_eq!((-s1).0, -100);
        assert_eq!((s1 * 2).0, 200);
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
            let s1 = Score(a);
            let s2 = Score(b);
            let _ = s1 + s2;
            let _ = s1 - s2;
            let _ = -s1;
        }
    }
}
