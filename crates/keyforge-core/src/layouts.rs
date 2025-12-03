use crate::keycodes::KeycodeRegistry;
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

/// Converts a layout string into u16 codes.
/// Unknown tokens in brackets `[MACRO]` are hashed into the 0xE000-0xEFFF range.
pub fn layout_string_to_u16(s: &str, size: usize, registry: &KeycodeRegistry) -> Vec<u16> {
    let mut codes = Vec::with_capacity(size);

    if s.trim().contains(' ') {
        for token in s.split_whitespace() {
            if codes.len() >= size {
                break;
            }

            if let Some(code) = registry.get_code(token) {
                codes.push(code);
            } else {
                // Dynamic Hash for custom tokens
                if token.starts_with('[') && token.ends_with(']') {
                    let content = &token[1..token.len() - 1];
                    let mut hasher = FnvHasher::default();
                    content.to_uppercase().hash(&mut hasher);
                    let hash = hasher.finish();
                    // Map to 0xE000 - 0xEFFF (4096 slots) - Low collision risk
                    let dynamic_code = 0xE000 + (hash % 4096) as u16;
                    codes.push(dynamic_code);
                } else {
                    codes.push(0); // KC_NO
                }
            }
        }
    } else {
        // Fallback char parsing
        let mut chars = s.chars().peekable();
        while codes.len() < size {
            if let Some(c) = chars.next() {
                if c == '[' {
                    let mut token = String::new();
                    let mut closed = false;
                    while let Some(&next_c) = chars.peek() {
                        chars.next();
                        if next_c == ']' {
                            closed = true;
                            break;
                        }
                        token.push(next_c);
                    }
                    if closed {
                        if let Some(code) = registry.get_code(&token) {
                            codes.push(code);
                        } else {
                            let mut hasher = FnvHasher::default();
                            token.to_uppercase().hash(&mut hasher);
                            let hash = hasher.finish();
                            let dynamic_code = 0xE000 + (hash % 4096) as u16;
                            codes.push(dynamic_code);
                        }
                    } else {
                        codes.push(0);
                    }
                } else {
                    let mut buf = [0; 4];
                    let s_char = c.encode_utf8(&mut buf);
                    if let Some(code) = registry.get_code(s_char) {
                        codes.push(code);
                    } else {
                        codes.push(0);
                    }
                }
            } else {
                codes.push(0);
            }
        }
    }

    while codes.len() < size {
        codes.push(0);
    }

    codes
}
