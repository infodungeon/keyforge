use keyforge_model::constants::MAX_KEYBOARD_KEYS;
use crate::kernel::types::KeyCode;
// use std::collections::HashMap; // Removed unused import

pub(crate) struct PosMap<'a> {
    pub(crate) starts: &'a [u16],
    pub(crate) counts: &'a [u8],
    pub(crate) indices: &'a [u16],
    pub(crate) used_keys: &'a [u16],
}

impl<'a> PosMap<'a> {
    /// Creates a PosMap by manually populating the provided scratch buffers.
    /// This avoids large array initialization on every call.
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

        Self { starts, counts, indices, used_keys: used_keys_scratch }
    }

    #[inline(always)]
    pub(crate) fn get(&self, code: usize) -> &[u16] {
        if code >= 65536 { return &[]; }
        let start = self.starts[code] as usize;
        let count = self.counts[code] as usize;
        if count == 0 { return &[]; }
        &self.indices[start..start + count]
    }
}

/// Scratch space for physics operations to avoid re-allocating large arrays.
pub struct PhysicsScratch {
    pub(crate) starts: [u16; 65536],
    pub(crate) counts: [u8; 65536],
    pub(crate) indices: [u16; MAX_KEYBOARD_KEYS],
    pub(crate) current_offsets: [u8; 65536], // Helper buffer
    pub(crate) used_keys: Vec<u16>,
    pub(crate) char_usage: [f32; 65536],
}

impl Default for PhysicsScratch {
    fn default() -> Self {
        Self {
            starts: [0; 65536],
            counts: [0; 65536],
            indices: [0; MAX_KEYBOARD_KEYS],
            current_offsets: [0; 65536],
            used_keys: Vec::with_capacity(MAX_KEYBOARD_KEYS),
            char_usage: [0.0; 65536],
        }
    }
}

impl PhysicsScratch {
    /// Creates a new scratch instance.
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn clear_used(&mut self) {
        for &code in &self.used_keys {
            let c = code as usize;
            self.starts[c] = 0;
            self.counts[c] = 0;
            self.char_usage[c] = 0.0;
        }
    }
}