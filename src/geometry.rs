use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KeyNode {
    pub hand: u8,   // 0 = Left, 1 = Right
    pub finger: u8, // 0=Thumb, 1=Index, 2=Middle, 3=Ring, 4=Pinky
    pub row: i8,    // 0=Top, 1=Home, 2=Bottom
    pub col: i8,    // Visual column index
    pub x: f32,     // Physical X coordinate (units ~1u key width)
    pub y: f32,     // Physical Y coordinate

    #[serde(default)]
    pub is_stretch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardGeometry {
    pub keys: Vec<KeyNode>,
    pub prime_slots: Vec<usize>,
    pub med_slots: Vec<usize>,
    pub low_slots: Vec<usize>,
}

impl KeyboardGeometry {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Self {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("❌ Failed to read geometry file: {}", e));

        serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("❌ Failed to parse geometry JSON: {}", e))
    }

    /// Returns the standard 30-key row-staggered geometry (Legacy KEY_DEFS)
    pub fn standard() -> Self {
        let keys = vec![
            // ROW 0 (Top)
            KeyNode {
                hand: 0,
                finger: 4,
                row: 0,
                col: 0,
                x: 0.0,
                y: 0.0,
                is_stretch: false,
            }, // 0 Q
            KeyNode {
                hand: 0,
                finger: 3,
                row: 0,
                col: 1,
                x: 1.0,
                y: 0.0,
                is_stretch: false,
            }, // 1 W
            KeyNode {
                hand: 0,
                finger: 2,
                row: 0,
                col: 2,
                x: 2.0,
                y: 0.0,
                is_stretch: false,
            }, // 2 E
            KeyNode {
                hand: 0,
                finger: 1,
                row: 0,
                col: 3,
                x: 3.0,
                y: 0.0,
                is_stretch: false,
            }, // 3 R
            KeyNode {
                hand: 0,
                finger: 1,
                row: 0,
                col: 4,
                x: 4.0,
                y: 0.0,
                is_stretch: true,
            }, // 4 T
            KeyNode {
                hand: 1,
                finger: 1,
                row: 0,
                col: 5,
                x: 5.0,
                y: 0.0,
                is_stretch: true,
            }, // 5 Y
            KeyNode {
                hand: 1,
                finger: 1,
                row: 0,
                col: 6,
                x: 6.0,
                y: 0.0,
                is_stretch: false,
            }, // 6 U
            KeyNode {
                hand: 1,
                finger: 2,
                row: 0,
                col: 7,
                x: 7.0,
                y: 0.0,
                is_stretch: false,
            }, // 7 I
            KeyNode {
                hand: 1,
                finger: 3,
                row: 0,
                col: 8,
                x: 8.0,
                y: 0.0,
                is_stretch: false,
            }, // 8 O
            KeyNode {
                hand: 1,
                finger: 4,
                row: 0,
                col: 9,
                x: 9.0,
                y: 0.0,
                is_stretch: false,
            }, // 9 P
            // ROW 1 (Home)
            KeyNode {
                hand: 0,
                finger: 4,
                row: 1,
                col: 0,
                x: 0.2,
                y: 1.0,
                is_stretch: false,
            }, // 10 A
            KeyNode {
                hand: 0,
                finger: 3,
                row: 1,
                col: 1,
                x: 1.2,
                y: 1.0,
                is_stretch: false,
            }, // 11 S
            KeyNode {
                hand: 0,
                finger: 2,
                row: 1,
                col: 2,
                x: 2.2,
                y: 1.0,
                is_stretch: false,
            }, // 12 D
            KeyNode {
                hand: 0,
                finger: 1,
                row: 1,
                col: 3,
                x: 3.2,
                y: 1.0,
                is_stretch: false,
            }, // 13 F
            KeyNode {
                hand: 0,
                finger: 1,
                row: 1,
                col: 4,
                x: 4.2,
                y: 1.0,
                is_stretch: true,
            }, // 14 G
            KeyNode {
                hand: 1,
                finger: 1,
                row: 1,
                col: 5,
                x: 5.2,
                y: 1.0,
                is_stretch: true,
            }, // 15 H
            KeyNode {
                hand: 1,
                finger: 1,
                row: 1,
                col: 6,
                x: 6.2,
                y: 1.0,
                is_stretch: false,
            }, // 16 J
            KeyNode {
                hand: 1,
                finger: 2,
                row: 1,
                col: 7,
                x: 7.2,
                y: 1.0,
                is_stretch: false,
            }, // 17 K
            KeyNode {
                hand: 1,
                finger: 3,
                row: 1,
                col: 8,
                x: 8.2,
                y: 1.0,
                is_stretch: false,
            }, // 18 L
            KeyNode {
                hand: 1,
                finger: 4,
                row: 1,
                col: 9,
                x: 9.2,
                y: 1.0,
                is_stretch: false,
            }, // 19 ;
            // ROW 2 (Bottom)
            KeyNode {
                hand: 0,
                finger: 4,
                row: 2,
                col: 0,
                x: 0.5,
                y: 2.0,
                is_stretch: false,
            }, // 20 Z
            KeyNode {
                hand: 0,
                finger: 3,
                row: 2,
                col: 1,
                x: 1.5,
                y: 2.0,
                is_stretch: false,
            }, // 21 X
            KeyNode {
                hand: 0,
                finger: 2,
                row: 2,
                col: 2,
                x: 2.5,
                y: 2.0,
                is_stretch: false,
            }, // 22 C
            KeyNode {
                hand: 0,
                finger: 1,
                row: 2,
                col: 3,
                x: 3.5,
                y: 2.0,
                is_stretch: false,
            }, // 23 V
            KeyNode {
                hand: 0,
                finger: 1,
                row: 2,
                col: 4,
                x: 4.5,
                y: 2.0,
                is_stretch: true,
            }, // 24 B
            KeyNode {
                hand: 1,
                finger: 1,
                row: 2,
                col: 5,
                x: 5.5,
                y: 2.0,
                is_stretch: true,
            }, // 25 N
            KeyNode {
                hand: 1,
                finger: 1,
                row: 2,
                col: 6,
                x: 6.5,
                y: 2.0,
                is_stretch: false,
            }, // 26 M
            KeyNode {
                hand: 1,
                finger: 2,
                row: 2,
                col: 7,
                x: 7.5,
                y: 2.0,
                is_stretch: false,
            }, // 27 ,
            KeyNode {
                hand: 1,
                finger: 3,
                row: 2,
                col: 8,
                x: 8.5,
                y: 2.0,
                is_stretch: false,
            }, // 28 .
            KeyNode {
                hand: 1,
                finger: 4,
                row: 2,
                col: 9,
                x: 9.5,
                y: 2.0,
                is_stretch: false,
            }, // 29 /
        ];

        KeyboardGeometry {
            keys,
            prime_slots: vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
            med_slots: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            low_slots: vec![20, 21, 22, 23, 24, 25, 26, 27, 28, 29],
        }
    }
}
