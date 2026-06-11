//! TOML schema for one karaconf profile

use serde::Deserialize;
use std::collections::BTreeMap;

/// Keymap loaded from a `<name>.toml` file
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Profile {
    pub settings: Settings,
    pub macros: BTreeMap<String, MacroDef>,
    pub layers: BTreeMap<String, BTreeMap<String, LayerValue>>, // `base` is always active
}

/// Settings
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub os_layout: String, // contract for symbol shorthand; validated against `layouts::ALL`
    pub tap_time: u32,     // tap-hold window in ms
    pub combo_time: u32,   // combo simultaneous-press window in ms
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            os_layout: "qwerty-us".into(),
            tap_time: 100,
            combo_time: 50,
        }
    }
}

/// Macros
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MacroDef {
    Repeat { key: String, repeat: u32 }, // `{ key = "down_arrow", repeat = 5 }`
    Sequence(Vec<String>),               // `["hyphen", "shift+period"]`
    Text(String),                        // `"->"`, with `{...}` for non-character keys
}

/// Value in a layer table
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LayerValue {
    TapHold([String; 2]), // `[tap, hold]`; `hold` may name a layer (in base)
    Simple(String),       // direct key/macro, or layer-name trigger (in base)
}
