// libs/keyforge-infra/src/util/layout_parser.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_adapter::parsing::{parse_key, KeyAction};
use keyforge_model::{KeyCode, Layout};
use keyforge_model::keycodes::KeycodeRegistry;
use moka::sync::Cache;
use once_cell::sync::Lazy;
use std::time::Duration;
use tracing::warn;

// LRU Cache for parsed layouts
static LAYOUT_CACHE: Lazy<Cache<String, Layout>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(10_000)
        .time_to_live(Duration::from_secs(300))
        .build()
});

/// Parses a space-separated layout string into a `Layout` object, using an LRU cache for performance.
///
/// This "permissive" parser maps high-level semantic tokens (like `MT(MOD_LSFT, KC_ENT)`)
/// to physical keycodes using the provided registry. If a token is unknown, it falls back
/// to `KC_NO` instead of failing.
pub fn parse_layout_string_permissive_cached(
    s: &str,
    size: usize,
    registry: &KeycodeRegistry,
) -> Result<Layout, String> {
    let key = format!("{}:{}", size, s);

    if let Some(layout) = LAYOUT_CACHE.get(&key) {
        return Ok(layout);
    }

    let mut keys = Vec::with_capacity(size);
    let tokens: Vec<&str> = s.split_whitespace().collect();

    for token in tokens {
        if keys.len() >= size {
            break;
        }

        let action = parse_key(token);

        // Map semantic actions to physical keycodes for scoring
        let code = match action {
            KeyAction::Simple(s) => registry.get_code(&s),
            // For ModTap/LayerTap, we score the base key (tap action)
            KeyAction::ModTap { key, .. } => match *key {
                KeyAction::Simple(ref s) | KeyAction::Raw(ref s) => registry.get_code(s),
                _ => None,
            },
            KeyAction::LayerTap { key, .. } => match *key {
                KeyAction::Simple(ref s) | KeyAction::Raw(ref s) => registry.get_code(s),
                _ => None,
            },
            // Layer actions map to their codes if present
            KeyAction::LayerMomentary(_) => registry.get_code("MO"),
            KeyAction::LayerToggle(_) => registry.get_code("TG"),
            KeyAction::LayerOn(_) => registry.get_code("TO"),
            // Sticky mods
            KeyAction::StickyMod(m) => registry
                .get_code(&format!("OSM_{}", m))
                .or(registry.get_code("OSM")),
            KeyAction::Transparent => registry.get_code("KC_TRNS"),
            KeyAction::NoOp => registry.get_code("KC_NO"),
            KeyAction::CapsWord => registry.get_code("CAPS_WORD"),
            KeyAction::Raw(s) => registry.get_code(&s),
        };

        if let Some(c) = code {
            keys.push(c);
        } else {
            // Graceful Fallback: Log error, insert KC_NO (0).
            warn!("Unknown key token: '{}'. Swapping in KC_NO (0).", token);
            keys.push(KeyCode(0));
        }
    }

    // Pad with KC_NO if under-sized
    while keys.len() < size {
        keys.push(KeyCode(0));
    }

    let layout = Layout::new_unchecked(keys);
    LAYOUT_CACHE.insert(key, layout.clone());
    Ok(layout)
}
