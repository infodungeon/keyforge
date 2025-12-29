use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    Simple(String),
    Transparent,
    NoOp,
    LayerMomentary(u8),
    LayerToggle(u8),
    LayerOn(u8),
    ModTap { mod_name: String, key: String },
    LayerTap { layer: u8, key: String },
    StickyMod(String),
    CapsWord,
    Raw(String),
}

static MOD_TAP_RE: OnceLock<Regex> = OnceLock::new();
static LAYER_TAP_RE: OnceLock<Regex> = OnceLock::new();
static LAYER_ACTION_RE: OnceLock<Regex> = OnceLock::new();
static STICKY_MOD_RE: OnceLock<Regex> = OnceLock::new();

const MAX_LAYER: u8 = 31;
const MAX_TOKEN_LEN: usize = 32;

pub fn parse_key(token: &str) -> KeyAction {
    let t = token.trim();
    if t.len() > MAX_TOKEN_LEN {
        return KeyAction::Raw(t.chars().take(MAX_TOKEN_LEN).collect());
    }
    let upper = t.to_uppercase();

    if matches!(upper.as_str(), "TRNS" | "_______" | "_") { return KeyAction::Transparent; }
    if matches!(upper.as_str(), "NO" | "XXXXXXX" | "XXX") { return KeyAction::NoOp; }
    if matches!(upper.as_str(), "CAPS_WORD" | "CW") { return KeyAction::CapsWord; }

    let layer_re = LAYER_ACTION_RE.get_or_init(|| Regex::new(r"^(MO|TG|TO)\((\d+)\)$").expect("static regex"));
    if let Some(caps) = layer_re.captures(&upper) {
        if let (Some(action_match), Some(layer_match)) = (caps.get(1), caps.get(2)) {
            let action = action_match.as_str();
            let layer = layer_match.as_str().parse::<u8>().unwrap_or(0);
            if layer > MAX_LAYER { return KeyAction::Raw(t.to_string()); }
            return match action {
                "MO" => KeyAction::LayerMomentary(layer),
                "TG" => KeyAction::LayerToggle(layer),
                "TO" => KeyAction::LayerOn(layer),
                _ => KeyAction::Raw(t.to_string()),
            };
        }
    }

    let lt_re = LAYER_TAP_RE.get_or_init(|| Regex::new(r"^LT\((\d+),\s*(.+)\)$").expect("static regex"));
    if let Some(caps) = lt_re.captures(&upper) {
        if let (Some(layer_match), Some(key_match)) = (caps.get(1), caps.get(2)) {
            let layer = layer_match.as_str().parse::<u8>().unwrap_or(0);
            let key = key_match.as_str().trim().to_string();
            if layer > MAX_LAYER { return KeyAction::Raw(t.to_string()); }
            return KeyAction::LayerTap { layer, key };
        }
    }

    let mt_re = MOD_TAP_RE.get_or_init(|| Regex::new(r"^([A-Z0-9_]+)_T\((.+)\)$").expect("static regex"));
    if let Some(caps) = mt_re.captures(&upper) {
        if let (Some(mod_match), Some(key_match)) = (caps.get(1), caps.get(2)) {
            let mod_name = mod_match.as_str().to_string();
            let key = key_match.as_str().trim().to_string();
            return KeyAction::ModTap { mod_name, key };
        }
    }

    let sk_re = STICKY_MOD_RE.get_or_init(|| Regex::new(r"^(?:SK|OSM)\((.+)\)$").expect("static regex"));
    if let Some(caps) = sk_re.captures(&upper) {
        if let Some(mod_match) = caps.get(1) {
            return KeyAction::StickyMod(mod_match.as_str().to_string());
        }
    }

    if t.contains('(') || t.contains(')') { return KeyAction::Raw(t.to_string()); }
    if t.len() == 1 && t.chars().next().is_some_and(|c| c.is_alphanumeric()) {
        return KeyAction::Simple(format!("KC_{}", upper));
    }
    KeyAction::Simple(t.to_string())
}
