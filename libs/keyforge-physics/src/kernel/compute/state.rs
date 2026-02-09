use crate::kernel::types::KeyCode;
use keyforge_model::constants::{MAX_KEYBOARD_KEYS, MAX_KEYCODE_SPACE};

thread_local! {
    /// Global thread-local scratch buffer to avoid re-allocation per call.
    static SCRATCH: std::cell::RefCell<Option<PhysicsScratch>> = const { std::cell::RefCell::new(None) };
}

/// Executes a closure with access to the thread-local `PhysicsScratch`.
///
/// This ensures efficient reuse of the scratch buffer across different
/// scoring and analysis functions on the same thread.
///
/// # Errors
/// Returns `PhysicsError::Config` if scratch initialization fails.
pub fn with_scratch<F, R>(f: F) -> Result<R, crate::error::PhysicsError>
where
    F: FnOnce(&mut PhysicsScratch) -> R,
{
    SCRATCH.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            *opt = Some(PhysicsScratch::try_new()?);
        }
        Ok(f(opt.as_mut().ok_or_else(|| {
            crate::error::PhysicsError::Config("Scratch initialization failed".into())
        })?))
    })
}

use keyforge_model::types::KeyIndex;

#[derive(Debug)]
pub enum PosMap<'a> {
    /// A structured view into scratch buffers.
    Structured {
        starts: &'a [u16; MAX_KEYCODE_SPACE],
        counts: &'a [u8; MAX_KEYCODE_SPACE],
        indices: &'a [KeyIndex],
        used_keys: &'a [KeyCode],
    },
    /// A flat, direct mapping (idx = keycode, val = pos).
    /// Used by the Exact engine which doesn't maintain complex indices.
    Flat {
        map: &'a [KeyIndex],
        key_count: usize,
    },
}

impl<'a> PosMap<'a> {
    /// Creates a `PosMap` by manually populating the provided scratch buffers.
    pub(crate) fn from_scratch(
        layout: &[KeyCode],
        key_count: usize,
        starts: &'a mut [u16; MAX_KEYCODE_SPACE],
        counts: &'a mut [u8; MAX_KEYCODE_SPACE],
        indices: &'a mut [KeyIndex],
        current_offsets: &mut [u8; MAX_KEYCODE_SPACE],
        used_keys_scratch: &'a mut Vec<KeyCode>,
    ) -> Self {
        let limit = layout.len().min(key_count);

        // 1. Collect unique keys present in the layout
        used_keys_scratch.clear();
        for &code in layout.iter().take(limit) {
            used_keys_scratch.push(code);
        }
        used_keys_scratch.sort_unstable_by_key(|k| k.raw());
        used_keys_scratch.dedup();

        // 2. Clear counts for used keys
        for &code in used_keys_scratch.iter() {
            counts[code.as_usize()] = 0;
            current_offsets[code.as_usize()] = 0;
        }

        // 3. Count occurrences
        for &code in layout.iter().take(limit) {
            counts[code.as_usize()] += 1;
        }

        // 4. Calculate starts (prefix sum)
        let mut offset: usize = 0;
        for &code in used_keys_scratch.iter() {
            let c = code.as_usize();
            starts[c] = u16::try_from(offset).unwrap_or(u16::MAX);
            offset += usize::from(counts[c]);
        }

        // 5. Fill indices
        // Reset current_offsets and counts for used keys
        for &code in used_keys_scratch.iter() {
            current_offsets[code.as_usize()] = 0;
        }

        for (i, &code) in layout.iter().enumerate().take(limit) {
            let c_raw = code.as_usize();
            let base = usize::from(starts[c_raw]);
            let off = usize::from(current_offsets[c_raw]);
            indices[base + off] = KeyIndex::new(u16::try_from(i).unwrap_or(0));
            current_offsets[c_raw] += 1;
        }

        Self::Structured {
            starts: &*starts,
            counts: &*counts,
            indices: &*indices,
            used_keys: used_keys_scratch,
        }
    }

    /// Creates a `PosMap` from a flat position map slice.
    #[must_use]
    pub fn from_slice(map: &'a [KeyIndex], key_count: usize) -> Self {
        Self::Flat { map, key_count }
    }

    #[inline]
    #[must_use]
    pub fn get(&self, code: KeyCode) -> &[KeyIndex] {
        let code_raw = code.as_usize();
        match self {
            Self::Structured {
                starts,
                counts,
                indices,
                ..
            } => {
                if code_raw >= MAX_KEYCODE_SPACE {
                    return &[];
                }
                let start = usize::from(starts[code_raw]);
                let count = usize::from(counts[code_raw]);
                if count == 0 {
                    return &[];
                }
                &indices[start..start + count]
            }
            Self::Flat { map, key_count } => {
                if code_raw >= map.len() {
                    return &[];
                }
                let pos = map[code_raw];
                if pos.as_usize() >= *key_count {
                    &[]
                } else {
                    // Safety: map[code] is a single value, but we need a slice.
                    // Since it's a flat map, each key is assumed to have exactly one position.
                    // This is used by the Exact engine.
                    std::slice::from_ref(&map[code_raw])
                }
            }
        }
    }

    /// Returns the list of unique keys present in the layout.
    #[must_use]
    pub fn used_keys(&self) -> &[KeyCode] {
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
    pub(crate) indices: Box<[KeyIndex; MAX_KEYBOARD_KEYS]>,
    pub(crate) current_offsets: Box<[u8; MAX_KEYCODE_SPACE]>, // Helper buffer
    pub(crate) used_keys: Vec<KeyCode>,
    pub(crate) char_usage: Box<[u64; MAX_KEYCODE_SPACE]>,
    /// A flat mapping for SIMD kernels (`KeyCode` -> `KeyIndex`).
    pub(crate) flat_map: Box<[KeyIndex; MAX_KEYCODE_SPACE]>,
}

impl PhysicsScratch {
    /// Safely creates a new scratch instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::Config` if allocation or sizing fails.
    pub fn try_new() -> Result<Self, crate::error::PhysicsError> {
        let err = || crate::error::PhysicsError::Config("Scratch layout mismatch".into());

        Ok(Self {
            starts: vec![0u16; MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .map_err(|_| err())?,
            counts: vec![0u8; MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .map_err(|_| err())?,
            indices: vec![KeyIndex::new(0); MAX_KEYBOARD_KEYS]
                .into_boxed_slice()
                .try_into()
                .map_err(|_| err())?,
            current_offsets: vec![0u8; MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .map_err(|_| err())?,
            used_keys: Vec::with_capacity(MAX_KEYBOARD_KEYS),
            char_usage: vec![0u64; MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .map_err(|_| err())?,
            flat_map: vec![KeyIndex::new(u16::MAX); MAX_KEYCODE_SPACE]
                .into_boxed_slice()
                .try_into()
                .map_err(|_| err())?,
        })
    }
}

impl PhysicsScratch {
    pub(crate) fn clear_used(&mut self) {
        for &code in &self.used_keys {
            let c = code.as_usize();
            self.starts[c] = 0;
            self.counts[c] = 0;
            self.char_usage[c] = 0;
            self.flat_map[c] = KeyIndex::new(u16::MAX);
        }
    }

    /// Returns mutable references to the individual scratch buffers.
    /// This allows bypassing the single mutable borrow restriction on the parent struct.
    #[allow(clippy::type_complexity)]
    pub(crate) fn get_mut_scratch(
        &mut self,
    ) -> (
        &mut [u16; MAX_KEYCODE_SPACE],
        &mut [u8; MAX_KEYCODE_SPACE],
        &mut [KeyIndex],
        &mut [u8; MAX_KEYCODE_SPACE],
        &mut Vec<KeyCode>,
        &mut [u64; MAX_KEYCODE_SPACE],
        &mut [KeyIndex; MAX_KEYCODE_SPACE],
    ) {
        (
            self.starts.as_mut(),
            self.counts.as_mut(),
            self.indices.as_mut_slice(),
            self.current_offsets.as_mut(),
            &mut self.used_keys,
            self.char_usage.as_mut(),
            self.flat_map.as_mut(),
        )
    }
}
