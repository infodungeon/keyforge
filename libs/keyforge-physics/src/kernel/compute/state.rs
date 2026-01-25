use crate::kernel::types::KeyCode;
use keyforge_model::constants::{MAX_KEYBOARD_KEYS, MAX_KEYCODE_SPACE};
use std::cell::RefCell;

thread_local! {
    /// Global thread-local scratch buffer to avoid re-allocation per call.
    /// Centralized to ensure all engines and analysis tools share the same memory.
    static SCRATCH: RefCell<PhysicsScratch> = RefCell::new(PhysicsScratch::new());
}

/// Executes a closure with access to the thread-local `PhysicsScratch`.
///
/// This ensures efficient reuse of the scratch buffer across different
/// scoring and analysis functions on the same thread.
pub fn with_scratch<F, R>(f: F) -> R
where
    F: FnOnce(&mut PhysicsScratch) -> R,
{
    SCRATCH.with(|cell| {
        let mut scratch = cell.borrow_mut();
        // We do NOT automatically clear here because some callers might need to inspect
        // the state (e.g. debugging) or they handle clearing themselves (like `score_layout`).
        // Ideally, the caller should clear if they dirty it, but `PhysicsScratch::clear_used`
        // is efficient.
        f(&mut scratch)
    })
}

#[derive(Debug)]
pub enum PosMap<'a> {
    /// A structured view into scratch buffers.
    Structured {
        starts: &'a [u16],
        counts: &'a [u8],
        indices: &'a [u16],
        used_keys: &'a [u16],
    },
    /// A flat, direct mapping (idx = keycode, val = pos).
    /// Used by the Exact engine which doesn't maintain complex indices.
    Flat { map: &'a [u16], key_count: usize },
}

#[allow(clippy::cast_possible_truncation)]
impl<'a> PosMap<'a> {
    /// Creates a `PosMap` by manually populating the provided scratch buffers.
    pub(crate) fn from_scratch(
        layout: &[KeyCode],
        key_count: usize,
        starts: &'a mut [u16],
        counts: &'a mut [u8],
        indices: &'a mut [u16],
        current_offsets: &mut [u8],
        used_keys_scratch: &'a mut Vec<u16>,
    ) -> Self {
        let limit = layout.len().min(key_count);

        // 1. Collect unique keys present in the layout
        used_keys_scratch.clear();
        for &code in layout.iter().take(limit) {
            used_keys_scratch.push(code.0);
        }
        used_keys_scratch.sort_unstable();
        used_keys_scratch.dedup();

        // 2. Clear counts for used keys
        for &code in used_keys_scratch.iter() {
            counts[code as usize] = 0;
        }

        // 3. Count occurrences
        for &code in layout.iter().take(limit) {
            counts[code.0 as usize] += 1;
        }

        // 4. Calculate starts (prefix sum)
        let mut offset = 0;
        for &code in used_keys_scratch.iter() {
            let c = code as usize;
            starts[c] = offset as u16;
            offset += counts[c] as usize;
        }

        // 5. Fill indices
        // Reset current_offsets
        for &code in used_keys_scratch.iter() {
            current_offsets[code as usize] = 0;
        }

        for (i, &code) in layout.iter().enumerate().take(limit) {
            let c = code.0 as usize;
            let base = starts[c] as usize;
            let off = current_offsets[c] as usize;
            indices[base + off] = i as u16;
            current_offsets[c] += 1;
        }

        Self::Structured {
            starts,
            counts,
            indices,
            used_keys: used_keys_scratch,
        }
    }

    /// Creates a `PosMap` from a flat position map slice.
    #[must_use]
    pub fn from_slice(map: &'a [u16], key_count: usize) -> Self {
        Self::Flat { map, key_count }
    }

    #[inline]
    #[must_use]
    pub fn get(&self, code: usize) -> &[u16] {
        match self {
            Self::Structured {
                starts,
                counts,
                indices,
                ..
            } => {
                if code >= MAX_KEYCODE_SPACE {
                    return &[];
                }
                let start = starts[code] as usize;
                let count = counts[code] as usize;
                if count == 0 {
                    return &[];
                }
                &indices[start..start + count]
            }
            Self::Flat { map, key_count } => {
                if code >= map.len() {
                    return &[];
                }
                let pos = map[code];
                if pos as usize >= *key_count {
                    &[]
                } else {
                    // Safety: map[code] is a single value, but we need a slice.
                    // Since it's a flat map, each key is assumed to have exactly one position.
                    // This is used by the Exact engine.
                    std::slice::from_ref(&map[code])
                }
            }
        }
    }

    /// Returns the list of unique keys present in the layout.
    #[must_use]
    pub fn used_keys(&self) -> &[u16] {
        match self {
            Self::Structured { used_keys, .. } => used_keys,
            Self::Flat { .. } => &[], // Not easily available for flat variant without scanning
        }
    }
}

/// Scratch space for physics operations to avoid re-allocating large arrays.
#[derive(Debug)]
pub struct PhysicsScratch {
    pub(crate) starts: Box<[u16; MAX_KEYCODE_SPACE]>,
    pub(crate) counts: Box<[u8; MAX_KEYCODE_SPACE]>,
    pub(crate) indices: Box<[u16; MAX_KEYBOARD_KEYS]>,
    pub(crate) current_offsets: Box<[u8; MAX_KEYCODE_SPACE]>, // Helper buffer
    pub(crate) used_keys: Vec<u16>,
    pub(crate) char_usage: Box<[f32; MAX_KEYCODE_SPACE]>,
    /// A flat mapping for SIMD kernels (`KeyCode` -> `KeyIndex`).
    pub(crate) flat_map: Box<[u16; MAX_KEYCODE_SPACE]>,
}

impl Default for PhysicsScratch {
    fn default() -> Self {
        #[allow(clippy::unwrap_used)]
        Self {
            starts: vec![0u16; MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            counts: vec![0u8; MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            indices: vec![0u16; MAX_KEYBOARD_KEYS]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            current_offsets: vec![0u8; MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            used_keys: Vec::with_capacity(MAX_KEYBOARD_KEYS),
            char_usage: vec![0.0f32; MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            flat_map: vec![u16::MAX; MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        }
    }
}

impl PhysicsScratch {
    /// Creates a new scratch instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn clear_used(&mut self) {
        for &code in &self.used_keys {
            let c = code as usize;
            self.starts[c] = 0;
            self.counts[c] = 0;
            self.char_usage[c] = 0.0;
            self.flat_map[c] = u16::MAX;
        }
    }

    /// Returns mutable references to the individual scratch buffers.
    /// This allows bypassing the single mutable borrow restriction on the parent struct.
    #[allow(clippy::type_complexity)]
    pub(crate) fn get_mut_scratch(
        &mut self,
    ) -> (
        &mut [u16],
        &mut [u8],
        &mut [u16],
        &mut [u8],
        &mut Vec<u16>,
        &mut [f32; MAX_KEYCODE_SPACE],
        &mut [u16; MAX_KEYCODE_SPACE],
    ) {
        (
            self.starts.as_mut_slice(),
            self.counts.as_mut_slice(),
            self.indices.as_mut_slice(),
            self.current_offsets.as_mut_slice(),
            &mut self.used_keys,
            self.char_usage.as_mut(),
            self.flat_map.as_mut(),
        )
    }
}
