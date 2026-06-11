use crate::karabiner::{ToEvent, ToKey};

/// Symbol-shorthand table for a given physical keyboard layout
#[derive(Debug)]
pub struct Layout {
    pub unshifted: &'static [(&'static str, &'static str)], // resolve with no modifier
    pub shifted: &'static [(&'static str, &'static str)],   // resolve with `["shift"]`
}

impl Layout {
    /// Look up by name in `layouts::ALL`. Errors with the available list.
    pub fn for_keyboard(name: &str) -> Result<&'static Layout, String> {
        crate::layouts::ALL
            .iter()
            .find_map(|&(n, l)| (n == name).then_some(l))
            .ok_or_else(|| {
                let available: Vec<&str> = crate::layouts::ALL.iter().map(|&(n, _)| n).collect();
                format!(
                    "unsupported os_layout {name:?}. Available: {available:?}. \
                     Symbol shorthand (#, +, |, …) needs a known layout; on other \
                     layouts, write the underlying key directly (e.g. \"shift+4\" \
                     instead of \"$\")."
                )
            })
    }

    /// Symbol shorthand (`#`, `+`, `|`, …) -> (key_code, modifiers).
    fn resolve_symbol(&self, expr: &str) -> Option<(String, Vec<String>)> {
        if let Some(&(_, k)) = self.unshifted.iter().find(|(s, _)| *s == expr) {
            Some((k.into(), vec![]))
        } else {
            self.shifted
                .iter()
                .find(|(s, _)| *s == expr)
                .map(|&(_, k)| (k.into(), vec!["shift".into()]))
        }
    }
}

/// Key-press event from an expression like `"shift+period"`
pub fn key_event(layout: &Layout, expr: &str) -> Result<ToEvent, String> {
    let (key_code, modifiers) = parse_key_expr(layout, expr)?;
    Ok(ToEvent::Key(ToKey {
        key_code,
        modifiers,
    }))
}

/// Plain key code (no modifiers). For contexts that can't carry modifiers,
/// like a combo part.
pub fn key_code_only(layout: &Layout, expr: &str) -> Result<String, String> {
    let (key_code, modifiers) = parse_key_expr(layout, expr)?;
    if !modifiers.is_empty() {
        return Err(format!(
            "{expr:?} must be a plain key code (no modifiers)"
        ));
    }
    Ok(key_code)
}

/// Text -> events
pub fn expand_text(layout: &Layout, text: &str) -> Result<Vec<ToEvent>, String> {
    let mut events = Vec::new();
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut expr = String::new();
            let mut closed = false;
            for c in chars.by_ref() {
                if c == '}' {
                    closed = true;
                    break;
                }
                expr.push(c);
            }
            if !closed {
                return Err(format!("unclosed '{{' in text macro: {text:?}"));
            }
            events.push(key_event(layout, &expr)?);
        } else {
            let (key_code, modifiers) = char_to_key(layout, ch)?;
            events.push(ToEvent::Key(ToKey {
                key_code,
                modifiers,
            }));
        }
    }
    Ok(events)
}

/// Parse `mod+...+key_or_symbol`
fn parse_key_expr(layout: &Layout, expr: &str) -> Result<(String, Vec<String>), String> {
    if expr.is_empty() {
        return Err("empty key expression".into());
    }
    if let Some(resolved) = layout.resolve_symbol(expr) {
        return Ok(resolved);
    }

    let parts: Vec<&str> = if expr.ends_with("++") {
        let mut p: Vec<&str> = expr[..expr.len() - 2].split('+').collect();
        p.push("+");
        p
    } else {
        expr.split('+').collect()
    };

    if parts.iter().any(|p| p.is_empty()) {
        return Err(format!("empty part in key expression {expr:?}"));
    }

    let last = *parts.last().unwrap();
    let prefix: Vec<String> = parts[..parts.len() - 1]
        .iter()
        .map(|s| (*s).into())
        .collect();

    if let Some((key, mut mods)) = layout.resolve_symbol(last) {
        for m in prefix {
            if !mods.contains(&m) {
                mods.push(m);
            }
        }
        Ok((key, mods))
    } else if looks_like_keycode(last) {
        Ok((last.into(), prefix))
    } else {
        Err(format!(
            "unknown key {last:?} in expression {expr:?} — not a Karabiner key code \
             or a US-QWERTY symbol shorthand. Use a key code (e.g. 'left_arrow', \
             'spacebar', 'f12') or a known symbol (#, +, |, …)."
        ))
    }
}

/// character to (key_code, modifiers)
fn char_to_key(layout: &Layout, ch: char) -> Result<(String, Vec<String>), String> {
    if let Some(resolved) = layout.resolve_symbol(&ch.to_string()) {
        return Ok(resolved);
    }
    Ok(match ch {
        'a'..='z' | '0'..='9' => (ch.to_string(), vec![]),
        'A'..='Z' => (ch.to_ascii_lowercase().to_string(), vec!["shift".into()]),
        ' ' => ("spacebar".into(), vec![]),
        _ => {
            return Err(format!(
                "unknown character {ch:?} in text macro — only ASCII letters/digits, \
             space, and US-QWERTY symbols are recognized. For other keys, embed \
             a key expression in {{...}} braces (e.g. {{home}}, {{shift+down_arrow}})."
            ))
        }
    })
}

fn looks_like_keycode(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> &'static Layout {
        Layout::for_keyboard("qwerty-us").unwrap()
    }

    #[test]
    fn parse_key_simple() {
        assert_eq!(
            parse_key_expr(layout(), "left_arrow").unwrap(),
            ("left_arrow".into(), vec![])
        );
    }

    #[test]
    fn parse_key_with_modifier() {
        assert_eq!(
            parse_key_expr(layout(), "shift+period").unwrap(),
            ("period".into(), vec!["shift".into()])
        );
    }

    #[test]
    fn parse_key_multi_modifier() {
        assert_eq!(
            parse_key_expr(layout(), "shift+option+left_arrow").unwrap(),
            ("left_arrow".into(), vec!["shift".into(), "option".into()]),
        );
    }

    #[test]
    fn parse_key_symbol_shifted() {
        assert_eq!(
            parse_key_expr(layout(), "#").unwrap(),
            ("3".into(), vec!["shift".into()])
        );
    }

    #[test]
    fn parse_key_symbol_unshifted() {
        assert_eq!(
            parse_key_expr(layout(), "/").unwrap(),
            ("slash".into(), vec![])
        );
    }

    #[test]
    fn parse_key_symbol_pipe() {
        assert_eq!(
            parse_key_expr(layout(), "|").unwrap(),
            ("backslash".into(), vec!["shift".into()])
        );
    }

    #[test]
    fn parse_key_modifier_plus_symbol() {
        // `option+#` = option + (shift+3)
        assert_eq!(
            parse_key_expr(layout(), "option+#").unwrap(),
            ("3".into(), vec!["shift".into(), "option".into()]),
        );
    }

    #[test]
    fn parse_key_trailing_plus_is_literal_plus() {
        // `command++` = Cmd+Plus = Cmd+Shift+= on US QWERTY.
        assert_eq!(
            parse_key_expr(layout(), "command++").unwrap(),
            ("equal_sign".into(), vec!["shift".into(), "command".into()]),
        );
    }

    #[test]
    fn parse_key_empty_part_errors() {
        assert!(parse_key_expr(layout(), "command+").is_err());
        assert!(parse_key_expr(layout(), "+command").is_err());
        assert!(parse_key_expr(layout(), "a++b").is_err());
        assert!(parse_key_expr(layout(), "").is_err());
    }

    #[test]
    fn parse_key_unknown_shape_errors() {
        // Single non-ASCII char is neither a known symbol nor a valid keycode shape
        let err = parse_key_expr(layout(), "§").unwrap_err();
        assert!(err.contains("§"), "got: {err}");
        assert!(parse_key_expr(layout(), "control+§").is_err());
        // 'left arrow' (with a space) doesn't match the keycode shape either
        assert!(parse_key_expr(layout(), "left arrow").is_err());
    }

    #[test]
    fn char_to_key_letter() {
        assert_eq!(char_to_key(layout(), 'a').unwrap(), ("a".into(), vec![]));
    }

    #[test]
    fn char_to_key_uppercase() {
        assert_eq!(
            char_to_key(layout(), 'A').unwrap(),
            ("a".into(), vec!["shift".into()])
        );
    }

    #[test]
    fn char_to_key_space() {
        assert_eq!(
            char_to_key(layout(), ' ').unwrap(),
            ("spacebar".into(), vec![])
        );
    }

    #[test]
    fn char_to_key_unknown_errors() {
        assert!(char_to_key(layout(), '→').is_err());
    }

    #[test]
    fn expand_text_arrow() {
        assert_eq!(expand_text(layout(), "->").unwrap().len(), 2);
    }

    #[test]
    fn expand_text_with_special_keys() {
        assert_eq!(
            expand_text(layout(), "{home}{shift+down_arrow}")
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn expand_text_mixed() {
        // g, i, t, spacebar, c, o, m, m, i, t = 10
        assert_eq!(
            expand_text(layout(), "git{spacebar}commit").unwrap().len(),
            10
        );
    }

    #[test]
    fn expand_text_unclosed_brace_errors() {
        assert!(expand_text(layout(), "hello{home").is_err());
    }

    #[test]
    fn for_keyboard_unknown_lists_available() {
        let err = Layout::for_keyboard("nope").unwrap_err();
        assert!(err.contains("nope"));
        assert!(err.contains("qwerty-us"));
    }

    #[test]
    fn qwerty_us_layout_has_no_symbol_conflicts() {
        let l = layout();
        for &(sym, _) in l.shifted {
            assert!(
                !l.unshifted.iter().any(|&(s, _)| s == sym),
                "{sym:?} appears in both shifted and unshifted",
            );
        }
    }
}
