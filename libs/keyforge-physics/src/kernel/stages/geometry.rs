use super::CompilationStage;
use crate::error::PhysicsError;
use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, Movement, Point, RowIndex, Score};
use keyforge_model::{Keyboard, Rubric};

/// Intermediate state containing processed geometry and spatial math.
#[derive(Debug)]
pub(crate) struct GeometryOutput {
    pub hands: Vec<HandIndex>,
    pub fingers: Vec<FingerIndex>,
    pub rows: Vec<RowIndex>,
    pub cols: Vec<ColIndex>,
    pub dist_matrix: Vec<Score>,
    pub key_home_distances: Vec<Score>,
}

/// Stage 1: Geometry & Spatial Math.
#[derive(Debug)]
pub(crate) struct GeometryStage<'a> {
    pub rubric: &'a Rubric,
}

impl<'a> CompilationStage for GeometryStage<'a> {
    type Input = &'a Keyboard;
    type Output = GeometryOutput;

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn execute(&self, kb: Self::Input) -> Result<Self::Output, PhysicsError> {
        let key_count = kb.count();
        let t_lat = f64::from(self.rubric.travel_lat());
        let t_vert = f64::from(self.rubric.travel_vert());
        let mut hands = Vec::with_capacity(key_count);
        let mut fingers = Vec::with_capacity(key_count);
        let mut rows = Vec::with_capacity(key_count);
        let mut cols = Vec::with_capacity(key_count);
        let mut key_home_distances = Vec::with_capacity(key_count);

        for k in &*kb.keys {
            hands.push(k.hand);
            fingers.push(k.finger);
            rows.push(k.row);
            cols.push(k.col);

            let mut dist_from_home = Score::ZERO;
            if let Some(origin) = kb
                .finger_origins
                .get(k.hand.as_usize())
                .and_then(|h| h.get(k.finger.as_usize()))
            {
                let movement = Movement::from_points(*origin, Point::new(k.x, k.y));
                let dx2 = i64::from(movement.dx) * i64::from(movement.dx);
                let dy2 = i64::from(movement.dy) * i64::from(movement.dy);

                let d_val = (dx2 as f64 * t_lat + dy2 as f64 * t_vert).sqrt();
                dist_from_home = Score::from_f32(d_val as f32)
                    .map_err(|e| PhysicsError::InvalidInput { message: e })?;
            }
            key_home_distances.push(dist_from_home);
        }

        let mut dist_matrix = vec![Score::ZERO; key_count * key_count];
        for i in 0..key_count {
            for j in 0..key_count {
                if i != j {
                    let movement = kb.spatial_cache[i * key_count + j];
                    let dx2 = i64::from(movement.dx) * i64::from(movement.dx);
                    let dy2 = i64::from(movement.dy) * i64::from(movement.dy);

                    let d_val = (dx2 as f64 * t_lat + dy2 as f64 * t_vert).sqrt();
                    dist_matrix[i * key_count + j] = Score::from_f32(d_val as f32)
                        .map_err(|e| PhysicsError::InvalidInput { message: e })?;
                }
            }
        }

        Ok(GeometryOutput {
            hands,
            fingers,
            rows,
            cols,
            dist_matrix,
            key_home_distances,
        })
    }
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use keyforge_model::KeyNode;

    #[test]
    fn test_geometry_stage_execution() {
        let keys = vec![
            KeyNode {
                index: 0,
                x: keyforge_model::types::SpatialUnit::from_f32(0.0),
                y: keyforge_model::types::SpatialUnit::from_f32(0.0),
                hand: HandIndex::new(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                x: keyforge_model::types::SpatialUnit::from_f32(3.0),
                y: keyforge_model::types::SpatialUnit::from_f32(4.0),
                hand: HandIndex::new(0),
                finger: FingerIndex::new_unchecked(1),
                ..Default::default()
            },
        ];
        let kb = Keyboard::new(keys, RowIndex(0), "test".into()).unwrap();
        let rubric = Rubric::builder().travel_lat(1.0).travel_vert(1.0).build();
        let stage = GeometryStage { rubric: &rubric };
        let out = stage.execute(&kb).unwrap();

        assert_eq!(out.hands.len(), 2);
        // dist = sqrt((3*1000)^2 + (4*1000)^2) = sqrt(9M + 16M) = 5000 units = 5.0 KU.
        // Score::from_f32(5.0) -> 5,000,000 raw.
        assert_eq!(out.dist_matrix[1].to_f32(), 5.0);
    }
}
