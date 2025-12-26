//! Protocol parsing facade.
//!
//! In the hex architecture, parsing of protocol/keyboard key strings is considered a
//! boundary concern. Downstream crates should depend on this adapter module rather
//! than importing `keyforge_protocol::parsing` directly.

pub use keyforge_protocol::parsing::{parse_key, KeyAction};
