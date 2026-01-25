// libs/keyforge-adapter/src/utils/padding.rs

//! Utility for padding sequences to a fixed length.
//!
//! This is used to ensure consistent sequence lengths for physics kernels
//! that expect fixed-size inputs, ensuring bit-perfect determinism across
//! different frameworks (JAX, `PyTorch`) and tokenization boundaries.

/// Pads a sequence of values to the specified length using a padding value.
///
/// If the sequence is already longer than the target length, it is truncated.
pub fn pad_sequence<T: Clone>(seq: &[T], length: usize, pad_value: T) -> Vec<T> {
    let mut padded = seq.to_vec();
    if padded.len() > length {
        padded.truncate(length);
    } else {
        padded.resize(length, pad_value);
    }
    padded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding() {
        let seq = vec![1, 2, 3];
        assert_eq!(pad_sequence(&seq, 5, 0), vec![1, 2, 3, 0, 0]);
        assert_eq!(pad_sequence(&seq, 2, 0), vec![1, 2]);
        assert_eq!(pad_sequence(&seq, 3, 0), vec![1, 2, 3]);
    }
}
