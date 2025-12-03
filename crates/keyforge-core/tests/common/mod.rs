#![allow(dead_code)] // FIXED: Suppress warnings for unused test helpers

use keyforge_core::geometry::{KeyNode, KeyboardGeometry};

/// Builder for KeyNode to clean up tests
pub struct KeyBuilder {
    node: KeyNode,
}

impl KeyBuilder {
    pub fn new(row: i8, col: i8) -> Self {
        Self {
            node: KeyNode {
                id: format!("k_{}_{}", row, col),
                hand: if col < 5 { 0 } else { 1 },
                finger: 1, // Default to Index
                row,
                col,
                x: col as f32,
                y: row as f32,
                w: 1.0,
                h: 1.0,
                is_stretch: false,
            },
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.node.id = id.to_string();
        self
    }

    pub fn hand(mut self, hand: u8) -> Self {
        self.node.hand = hand;
        self
    }

    pub fn finger(mut self, finger: u8) -> Self {
        self.node.finger = finger;
        self
    }

    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.node.x = x;
        self.node.y = y;
        self
    }

    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.node.w = w;
        self.node.h = h;
        self
    }

    pub fn stretch(mut self, is_stretch: bool) -> Self {
        self.node.is_stretch = is_stretch;
        self
    }

    pub fn build(self) -> KeyNode {
        self.node
    }
}

/// Helper to quickly create a geometry from a list of keys
pub fn create_geom(keys: Vec<KeyNode>) -> KeyboardGeometry {
    let key_count = keys.len();
    let mut geom = KeyboardGeometry {
        keys,
        prime_slots: (0..key_count).collect(),
        med_slots: vec![],
        low_slots: vec![],
        home_row: 0,
        finger_origins: [[(0.0, 0.0); 5]; 2],
    };
    geom.calculate_origins();
    geom
}

/// Standard 30-key Ortho Mock
pub fn mock_ortho_30() -> KeyboardGeometry {
    let mut keys = Vec::new();
    for r in 0..3 {
        for c in 0..10 {
            keys.push(KeyBuilder::new(r, c).build());
        }
    }
    create_geom(keys)
}
