//! symbol shorthand tables

mod qwerty_us;

use crate::keys::Layout;

pub const ALL: &[(&str, &Layout)] = &[("qwerty-us", &qwerty_us::QWERTY_US)];
