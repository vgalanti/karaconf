//! Compile a `Profile` into Karabiner rules

use crate::config::{LayerValue, MacroDef, Profile};
use crate::karabiner::*;
use crate::keys::{expand_text, key_code_only, key_event, Layout};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Always-active layer; others are gated by triggers declared here
const BASE_LAYER: &str = "base";

/// Layer names that would shadow a Karabiner key code
const RESERVED_LAYER_NAMES: &[&str] = &[
    "shift",
    "left_shift",
    "right_shift",
    "control",
    "left_control",
    "right_control",
    "option",
    "left_option",
    "right_option",
    "command",
    "left_command",
    "right_command",
    "caps_lock",
    "fn",
    "tab",
    "escape",
    "return_or_enter",
    "spacebar",
    "space",
    "delete_or_backspace",
    "delete_forward",
    "home",
    "end",
    "page_up",
    "page_down",
    "up_arrow",
    "down_arrow",
    "left_arrow",
    "right_arrow",
];

/// Compile a profile to rules: validate -> expand macros -> compile non-base
/// layers (conditioned) -> compile base layer
pub fn convert(profile: &Profile) -> Result<Vec<Rule>, String> {
    let layout = Layout::for_keyboard(&profile.settings.os_layout)?;
    validate_layer_names(profile)?;
    let macros = expand_macros(layout, &profile.macros)?;
    let layer_names: HashSet<&str> = profile
        .layers
        .keys()
        .filter(|k| k.as_str() != BASE_LAYER)
        .map(String::as_str)
        .collect();

    let mut rules = Vec::new();

    // Non-base layers first so their conditioned rules win first-match order
    for (name, keys) in &profile.layers {
        if name == BASE_LAYER {
            continue;
        }
        let mut manipulators = Vec::new();
        // Combos before singles so the chord wins inside its threshold window
        for (k, v) in keys.iter().filter(|(k, _)| is_combo(k)) {
            manipulators.push(combo_manipulator(
                layout,
                k,
                v,
                Some(name),
                &layer_names,
                &macros,
                profile.settings.combo_time,
            )?);
        }
        for (k, v) in keys.iter().filter(|(k, _)| !is_combo(k)) {
            manipulators.push(layer_manipulator(layout, k, v, name, &macros)?);
        }
        if !manipulators.is_empty() {
            rules.push(Rule {
                description: format!("Layer: {name}"),
                manipulators,
            });
        }
    }

    if let Some(base) = profile.layers.get(BASE_LAYER) {
        let mut manipulators = Vec::new();
        for (k, v) in base.iter().filter(|(k, _)| is_combo(k)) {
            manipulators.push(combo_manipulator(
                layout,
                k,
                v,
                None,
                &layer_names,
                &macros,
                profile.settings.combo_time,
            )?);
        }
        for (k, v) in base.iter().filter(|(k, _)| !is_combo(k)) {
            manipulators.push(base_manipulator(
                layout,
                k,
                v,
                &layer_names,
                &macros,
                profile.settings.tap_time,
            )?);
        }
        if !manipulators.is_empty() {
            rules.push(Rule {
                description: "Base".into(),
                manipulators,
            });
        }
    }

    Ok(rules)
}

fn is_combo(key: &str) -> bool {
    key.contains('+')
}

fn validate_layer_names(profile: &Profile) -> Result<(), String> {
    for name in profile.layers.keys() {
        if name == BASE_LAYER {
            continue;
        }
        let single_ascii_alphanum = name.len() == 1 && name.as_bytes()[0].is_ascii_alphanumeric();
        if single_ascii_alphanum || RESERVED_LAYER_NAMES.contains(&name.as_str()) {
            return Err(format!(
                "layer name '{name}' shadows a Karabiner key code; rename it (e.g. 'nav', 'sym')"
            ));
        }
    }
    Ok(())
}

/// Non-base-layer key. Gated by `variable_if`; tap-hold is base-only.
fn layer_manipulator(
    layout: &Layout,
    key: &str,
    value: &LayerValue,
    layer: &str,
    macros: &HashMap<String, Vec<ToEvent>>,
) -> Result<Manipulator, String> {
    match value {
        LayerValue::Simple(action) => Ok(Manipulator {
            to: resolve_action(layout, action, macros)?,
            conditions: vec![Condition::variable_if(layer_var(layer), 1)],
            ..Manipulator::from_key(key)
        }),
        LayerValue::TapHold(_) => Err(format!(
            "tap-hold in non-base layer '{layer}' for key '{key}' is not supported"
        )),
    }
}

/// Base-layer key. `Simple` = remap (or layer trigger if it names a layer)
/// `[tap, hold]` = tap-hold; `hold` may be a key or a layer name
fn base_manipulator(
    layout: &Layout,
    key: &str,
    value: &LayerValue,
    layer_names: &HashSet<&str>,
    macros: &HashMap<String, Vec<ToEvent>>,
    tap_time: u32,
) -> Result<Manipulator, String> {
    match value {
        LayerValue::Simple(action) => {
            if layer_names.contains(action.as_str()) {
                Ok(layer_trigger(key, action))
            } else {
                Ok(Manipulator {
                    to: resolve_action(layout, action, macros)?,
                    ..Manipulator::from_key(key)
                })
            }
        }
        LayerValue::TapHold([tap, hold]) => {
            // Build `hold`-side manipulator first then layer tap behavior on top
            let base = if layer_names.contains(hold.as_str()) {
                layer_trigger(key, hold)
            } else {
                Manipulator {
                    to: vec![key_event(layout, hold)?],
                    ..Manipulator::from_key(key)
                }
            };
            Ok(Manipulator {
                to_if_alone: vec![key_event(layout, tap)?],
                parameters: Some(Parameters {
                    to_if_alone_timeout: Some(tap_time),
                    simultaneous_threshold: None,
                }),
                ..base
            })
        }
    }
}

/// Combo key. `in_layer = Some(name)` adds the layer condition; layer-trigger
/// values are only honored in base (matches the single-key rules).
fn combo_manipulator(
    layout: &Layout,
    key: &str,
    value: &LayerValue,
    in_layer: Option<&str>,
    layer_names: &HashSet<&str>,
    macros: &HashMap<String, Vec<ToEvent>>,
    combo_time: u32,
) -> Result<Manipulator, String> {
    let parts = parse_combo(layout, key)?;
    let action = match value {
        LayerValue::Simple(a) => a,
        LayerValue::TapHold(_) => {
            return Err(format!("tap-hold is not supported for combo '{key}'"))
        }
    };
    let base = Manipulator::from_combo(&parts);
    let mut m = if in_layer.is_none() && layer_names.contains(action.as_str()) {
        let var = layer_var(action);
        Manipulator {
            to: vec![ToEvent::set_var(&var, 1)],
            to_after_key_up: vec![ToEvent::set_var(var, 0)],
            ..base
        }
    } else {
        Manipulator {
            to: resolve_action(layout, action, macros)?,
            ..base
        }
    };
    m.parameters = Some(Parameters {
        to_if_alone_timeout: None,
        simultaneous_threshold: Some(combo_time),
    });
    if let Some(layer) = in_layer {
        m.conditions = vec![Condition::variable_if(layer_var(layer), 1)];
    }
    Ok(m)
}

/// Split `"j+k"` into its key codes. Each part must resolve to a plain key.
fn parse_combo(layout: &Layout, expr: &str) -> Result<Vec<String>, String> {
    let parts: Vec<&str> = expr.split('+').collect();
    if parts.len() < 2 {
        return Err(format!("combo '{expr}' needs at least two keys"));
    }
    parts
        .iter()
        .map(|p| {
            if p.is_empty() {
                return Err(format!("empty part in combo '{expr}'"));
            }
            key_code_only(layout, p)
        })
        .collect()
}

/// flip `layer_<name>` on press / release.
fn layer_trigger(key: &str, layer: &str) -> Manipulator {
    let var = layer_var(layer);
    Manipulator {
        to: vec![ToEvent::set_var(&var, 1)],
        to_after_key_up: vec![ToEvent::set_var(var, 0)],
        ..Manipulator::from_key(key)
    }
}

/// `"$name"` -> macro expansion, else a single key event
fn resolve_action(
    layout: &Layout,
    action: &str,
    macros: &HashMap<String, Vec<ToEvent>>,
) -> Result<Vec<ToEvent>, String> {
    if let Some(name) = action.strip_prefix('$') {
        macros
            .get(name)
            .cloned()
            .ok_or_else(|| format!("macro '${name}' not found"))
    } else {
        Ok(vec![key_event(layout, action)?])
    }
}

/// Karabiner variable name for a layer's active flag.
fn layer_var(name: &str) -> String {
    format!("layer_{name}")
}

/// expand macros
fn expand_macros(
    layout: &Layout,
    macros: &BTreeMap<String, MacroDef>,
) -> Result<HashMap<String, Vec<ToEvent>>, String> {
    macros
        .iter()
        .map(|(n, d)| expand_macro(layout, d).map(|events| (n.clone(), events)))
        .collect()
}

fn expand_macro(layout: &Layout, def: &MacroDef) -> Result<Vec<ToEvent>, String> {
    match def {
        MacroDef::Text(text) => expand_text(layout, text),
        MacroDef::Sequence(steps) => steps.iter().map(|s| key_event(layout, s)).collect(),
        MacroDef::Repeat { key, repeat } => (0..*repeat).map(|_| key_event(layout, key)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> &'static Layout {
        Layout::for_keyboard("qwerty-us").unwrap()
    }

    #[test]
    fn expand_macro_repeat() {
        let events = expand_macro(
            layout(),
            &MacroDef::Repeat {
                key: "down_arrow".into(),
                repeat: 5,
            },
        )
        .unwrap();
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn expand_macro_sequence() {
        let events = expand_macro(
            layout(),
            &MacroDef::Sequence(vec!["hyphen".into(), "shift+period".into()]),
        )
        .unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn simple_value_matching_layer_name_is_a_trigger() {
        let toml = r#"
[layers.base]
right_command = "sym"

[layers.sym]
a = "="
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let json = serde_json::to_value(convert(&profile).unwrap()).unwrap();
        // rules[1] = "Base", manipulators[0] = right_command
        let m = &json[1]["manipulators"][0];
        assert_eq!(m["from"]["key_code"], "right_command");
        assert_eq!(m["to"][0]["set_variable"]["name"], "layer_sym");
        assert_eq!(m["to"][0]["set_variable"]["value"], 1);
        assert_eq!(m["to_after_key_up"][0]["set_variable"]["value"], 0);
        // No tap action and no timeout — it's pure layer-trigger.
        assert!(m.get("to_if_alone").is_none());
        assert!(m.get("parameters").is_none());
    }

    #[test]
    fn single_char_layer_name_is_rejected() {
        let toml = r#"[layers.s]
a = "b"
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let err = convert(&profile).unwrap_err();
        assert!(err.contains("'s'"));
    }

    #[test]
    fn reserved_layer_name_is_rejected() {
        let toml = r#"[layers.escape]
a = "b"
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let err = convert(&profile).unwrap_err();
        assert!(err.contains("escape"));
    }

    #[test]
    fn tap_hold_in_non_base_layer_errors() {
        let toml = r#"
[layers.base]
tab = ["tab", "nav"]
[layers.nav]
h = ["a", "b"]
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let err = convert(&profile).unwrap_err();
        assert!(err.contains("tap-hold"));
    }

    #[test]
    fn missing_macro_errors() {
        let toml = r#"
[layers.base]
quote = "$does_not_exist"
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let err = convert(&profile).unwrap_err();
        assert!(err.contains("does_not_exist"));
    }

    #[test]
    fn unknown_os_layout_error_is_informative() {
        let toml = r#"
[settings]
os_layout = "azerty-fr"
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let err = convert(&profile).unwrap_err();
        assert!(err.contains("azerty-fr"), "missing user input: {err}");
        assert!(err.contains("qwerty-us"), "missing supported list: {err}");
        assert!(err.contains("shift+4"), "missing workaround example: {err}");
    }

    #[test]
    fn combo_emits_simultaneous_and_threshold() {
        let toml = r#"
[settings]
combo_time = 40

[layers.base]
"j+k" = "escape"
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let json = serde_json::to_value(convert(&profile).unwrap()).unwrap();
        let m = &json[0]["manipulators"][0];
        assert_eq!(m["from"]["simultaneous"][0]["key_code"], "j");
        assert_eq!(m["from"]["simultaneous"][1]["key_code"], "k");
        assert!(m["from"].get("key_code").is_none());
        assert_eq!(m["to"][0]["key_code"], "escape");
        assert_eq!(
            m["parameters"]["basic.simultaneous_threshold_milliseconds"],
            40
        );
        assert!(m["parameters"]
            .get("basic.to_if_alone_timeout_milliseconds")
            .is_none());
    }

    #[test]
    fn combo_can_trigger_layer_in_base() {
        let toml = r#"
[layers.base]
"d+f" = "nav"

[layers.nav]
h = "left_arrow"
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let json = serde_json::to_value(convert(&profile).unwrap()).unwrap();
        // rules[1] = "Base", manipulators[0] = the combo (combos come first)
        let m = &json[1]["manipulators"][0];
        assert_eq!(m["from"]["simultaneous"][0]["key_code"], "d");
        assert_eq!(m["to"][0]["set_variable"]["name"], "layer_nav");
        assert_eq!(m["to"][0]["set_variable"]["value"], 1);
        assert_eq!(m["to_after_key_up"][0]["set_variable"]["value"], 0);
    }

    #[test]
    fn combo_in_non_base_layer_is_gated() {
        let toml = r#"
[layers.base]
tab = ["tab", "nav"]

[layers.nav]
"j+k" = "escape"
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let json = serde_json::to_value(convert(&profile).unwrap()).unwrap();
        let m = &json[0]["manipulators"][0];
        assert_eq!(m["from"]["simultaneous"][0]["key_code"], "j");
        assert_eq!(m["conditions"][0]["name"], "layer_nav");
    }

    #[test]
    fn combo_ordered_before_singles_in_same_rule() {
        let toml = r#"
[layers.base]
j = "x"
"j+k" = "escape"
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let json = serde_json::to_value(convert(&profile).unwrap()).unwrap();
        let ms = &json[0]["manipulators"];
        assert!(ms[0]["from"]["simultaneous"].is_array());
        assert_eq!(ms[1]["from"]["key_code"], "j");
    }

    #[test]
    fn combo_tap_hold_errors() {
        let toml = r#"
[layers.base]
"j+k" = ["escape", "left_control"]
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let err = convert(&profile).unwrap_err();
        assert!(err.contains("tap-hold"));
    }

    #[test]
    fn combo_single_part_errors() {
        let err = parse_combo(layout(), "j").unwrap_err();
        assert!(err.contains("at least two"));
    }

    #[test]
    fn combo_shifted_symbol_part_errors() {
        // `#` resolves to shift+3; combo parts can't carry modifiers
        let err = parse_combo(layout(), "j+#").unwrap_err();
        assert!(err.contains("modifiers"));
    }

    #[test]
    fn snapshot_full_conversion() {
        let toml = r#"
[layers.base]
caps_lock = ["escape", "left_control"]
tab = ["tab", "nav"]

[layers.nav]
h = "left_arrow"
"#;
        let profile: Profile = toml::from_str(toml).unwrap();
        let rules = convert(&profile).unwrap();
        let json = serde_json::to_string_pretty(&rules).unwrap();
        assert_eq!(json, include_str!("snapshot.json"));
    }
}
