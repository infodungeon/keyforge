use keyforge_adapter::parsing::{parse_key, KeyAction};
use keyforge_model::Layout;
use keyforge_protocol::keycodes::KeycodeRegistry;
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
            KeyAction::ModTap { key, .. } => registry.get_code(&key),
            KeyAction::LayerTap { key, .. } => registry.get_code(&key),
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
            keys.push(0);
        }
    }

    // Pad with KC_NO if under-sized
    while keys.len() < size {
        keys.push(0);
    }

    let layout = Layout::new_unchecked(keys);
    LAYOUT_CACHE.insert(key, layout.clone());
    Ok(layout)
}
