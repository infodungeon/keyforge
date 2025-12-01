// ===== keyforge/src/layouts.rs =====

/// Helper to convert a layout string (e.g. "QWERTY...") into a Vec<u8>
/// matching the specified size.
pub fn layout_string_to_bytes(s: &str, size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];

    // Standardize: Upper case, take first 'size' chars
    for (i, c) in s.to_uppercase().chars().take(size).enumerate() {
        // Simple ascii byte cast for now, assuming valid input
        bytes[i] = c as u8;
    }

    // Pad with space (32) if string is too short?
    // Or 0. Let's stick to 0 (NULL) which indicates "no key mapped"
    bytes
}
