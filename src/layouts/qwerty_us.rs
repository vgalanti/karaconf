//! us qwerty symbol -> key_code

use crate::keys::Layout;

pub const QWERTY_US: Layout = Layout {
    unshifted: &[
        ("-", "hyphen"),
        ("=", "equal_sign"),
        ("[", "open_bracket"),
        ("]", "close_bracket"),
        ("\\", "backslash"),
        (";", "semicolon"),
        ("'", "quote"),
        (",", "comma"),
        (".", "period"),
        ("/", "slash"),
        ("`", "grave_accent_and_tilde"),
    ],
    shifted: &[
        ("!", "1"),
        ("@", "2"),
        ("#", "3"),
        ("$", "4"),
        ("%", "5"),
        ("^", "6"),
        ("&", "7"),
        ("*", "8"),
        ("(", "9"),
        (")", "0"),
        ("_", "hyphen"),
        ("+", "equal_sign"),
        ("{", "open_bracket"),
        ("}", "close_bracket"),
        ("|", "backslash"),
        (":", "semicolon"),
        ("\"", "quote"),
        ("<", "comma"),
        (">", "period"),
        ("?", "slash"),
        ("~", "grave_accent_and_tilde"),
    ],
};
