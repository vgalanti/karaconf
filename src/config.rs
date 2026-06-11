//! TOML schema for one karaconf profile

use serde::Deserialize;
use std::collections::BTreeMap;

/// one `<name>.toml` profile
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Profile {
    pub settings: Settings,
    pub macros: BTreeMap<String, MacroDef>,
    pub layers: BTreeMap<String, BTreeMap<String, LayerValue>>, // `base` is always active
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub os_layout: String, // symbol-shorthand contract; validated against `layouts::ALL`
    pub tap_time: u32,     // tap-hold window, ms
    pub combo_time: u32,   // combo simultaneous-press window, ms
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MacroDef {
    Repeat { key: String, repeat: u32 }, // `{ key = "down_arrow", repeat = 5 }`
    Sequence(Vec<String>),               // `["hyphen", "shift+period"]`
    Text(String),                        // `"->"`; `{...}` for non-character keys
}

/// one layer-table value
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LayerValue {
    TapHold([String; 2]), // `[tap, hold]`; in base, `hold` may name a layer
    Simple(String),       // key/macro, or layer trigger in base
}
