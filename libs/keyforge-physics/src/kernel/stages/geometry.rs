use super::CompilationStage;
use crate::errors::PhysicsError;
use keyforge_model::types::{ColIndex, FingerIndex, HandIndex, RowIndex};
use keyforge_model::Keyboard;
use std::sync::Arc;

/// Intermediate state containing processed geometry and spatial math.
pub struct GeometryOutput {
    pub hands: Vec<HandIndex>,
    pub fingers: Vec<FingerIndex>,
    pub rows: Vec<RowIndex>,
    pub cols: Vec<ColIndex>,
    pub dist_matrix: Vec<f32>,
    pub key_home_distances: Vec<f32>,
}

/// Stage 1: Geometry & Spatial Math.
pub struct GeometryStage;

impl CompilationStage for GeometryStage {
    type Input = Arc<Keyboard>;
    type Output = GeometryOutput;

    fn execute(&self, kb: Self::Input) -> Result<Self::Output, PhysicsError> {
        let key_count = kb.count();
        let mut hands = Vec::with_capacity(key_count);
        let mut fingers = Vec::with_capacity(key_count);
        let mut rows = Vec::with_capacity(key_count);
        let mut cols = Vec::with_capacity(key_count);
        let mut key_home_distances = Vec::with_capacity(key_count);

        for k in &kb.keys {
            hands.push(k.hand);
            fingers.push(k.finger);
            rows.push(k.row);
            cols.push(k.col);

            let mut dist_from_home = 0.0;
            if let Some(origin) = kb
                .finger_origins
                .get(k.hand.as_usize())
                .and_then(|h| h.get(k.finger.as_usize()))
            {
                let dx = (k.x - origin.0).abs();
                let dy = (k.y - origin.1).abs();
                dist_from_home = (dx * dx + dy * dy).sqrt();
            }
            key_home_distances.push(dist_from_home);
        }

        let mut dist_matrix = vec![0.0f32; key_count * key_count];
        for i in 0..key_count {
            for j in 0..key_count {
                if i == j {
                    dist_matrix[i * key_count + j] = 0.0;
                } else {
                    let k1 = &kb.keys[i];
                    let k2 = &kb.keys[j];
                    let dx = (k1.x - k2.x).abs();
                    let dy = (k1.y - k2.y).abs();
                    dist_matrix[i * key_count + j] = (dx * dx + dy * dy).sqrt();
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

#[cfg(test)]
mod tests {
    use super::*;
    use keyforge_model::KeyNode;

    #[test]
    fn test_geometry_stage_execution() {
        let keys = vec![
            KeyNode {
                index: 0,
                x: 0.0,
                y: 0.0,
                hand: HandIndex(0),
                finger: FingerIndex(1),
                ..Default::default()
            },
            KeyNode {
                index: 1,
                x: 3.0,
                y: 4.0,
                hand: HandIndex(0),
                finger: FingerIndex(1),
                ..Default::default()
            },
        ];
        let kb = Arc::new(Keyboard::new(keys, 0).unwrap());
        let stage = GeometryStage;
        let out = stage.execute(kb).unwrap();

        assert_eq!(out.hands.len(), 2);
        // Distance from 0 to 1 should be sqrt(3^2 + 4^2) = 5
        assert_eq!(out.dist_matrix[1], 5.0);
    }
}
