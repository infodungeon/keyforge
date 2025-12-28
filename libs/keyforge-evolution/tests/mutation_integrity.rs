use keyforge_model::Layout;
use keyforge_evolution::supervisor::traits::MutationAction;
use proptest::prelude::*;

// Helper to generate a random layout of 30 unique keys [0..29]
fn arb_layout() -> impl Strategy<Value = Layout> {
    Just((0..30).collect::<Vec<u16>>()).prop_shuffle().prop_map(Layout::new)
}

proptest! {
    #[test]
    fn test_swap_conservation(
        mut layout in arb_layout(),
        a in 0usize..30,
        b in 0usize..30
    ) {
        let original_sum: u32 = layout.keys.iter().map(|&x| x as u32).sum();
        let mut pos_map = vec![65535u16; 256];
        
        // Initialize pos_map
        for (i, &code) in layout.keys.iter().enumerate() {
            if (code as usize) < pos_map.len() {
                pos_map[code as usize] = i as u16;
            }
        }

        let action = MutationAction::Swap(a, b);
        action.apply(&mut layout, &mut pos_map);

        let new_sum: u32 = layout.keys.iter().map(|&x| x as u32).sum();
        
        // Invariant 1: Conservation of keys
        assert_eq!(original_sum, new_sum, "Swap must not create or destroy keys");

        // Invariant 2: PosMap coherence
        for (i, &code) in layout.keys.iter().enumerate() {
            if (code as usize) < pos_map.len() {
                assert_eq!(pos_map[code as usize], i as u16, "PosMap must be coherent after swap");
            }
        }
    }

    #[test]
    fn test_group_swap_conservation(
        mut layout in arb_layout(),
        a in 0usize..30,
        b in 0usize..30,
        c in 0usize..30
    ) {
        let original_sum: u32 = layout.keys.iter().map(|&x| x as u32).sum();
        let mut pos_map = vec![65535u16; 256];
        
        for (i, &code) in layout.keys.iter().enumerate() {
            if (code as usize) < pos_map.len() {
                pos_map[code as usize] = i as u16;
            }
        }

        let action = MutationAction::GroupSwap(a, b, c);
        action.apply(&mut layout, &mut pos_map);

        let new_sum: u32 = layout.keys.iter().map(|&x| x as u32).sum();
        
        assert_eq!(original_sum, new_sum, "GroupSwap must not create or destroy keys");

        for (i, &code) in layout.keys.iter().enumerate() {
            if (code as usize) < pos_map.len() {
                assert_eq!(pos_map[code as usize], i as u16, "PosMap must be coherent after group swap");
            }
        }
    }
}

#[test]
fn test_swap_indices_same() {
    let mut layout = Layout::new(vec![1, 2, 3]);
    let mut pos_map = vec![65535u16; 10];
    pos_map[1] = 0;
    pos_map[2] = 1;
    pos_map[3] = 2;

    let action = MutationAction::Swap(1, 1);
    action.apply(&mut layout, &mut pos_map);

    assert_eq!(layout.keys.as_slice(), &[1, 2, 3]);
    assert_eq!(pos_map[2], 1);
}
