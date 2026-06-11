//! CLI commands + `karabiner.json` upserts - untouched profiles preserved

use crate::config::Profile;
use crate::converter;
use serde_json::{json, Value};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Reserved system default for resets
const SYSTEM_DEFAULT: &str = "system";

const KARABINER_CLI: &str =
    "/Library/Application Support/org.pqrs/Karabiner-Elements/bin/karabiner_cli";

/// Compile TOML files in `karaconf_dir` to upsert as a Karabiner profile
pub fn sync(karaconf_dir: &Path, karabiner_json: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(karaconf_dir)?;

    let mut karabiner = read_json(karabiner_json)?;
    let profiles = karabiner
        .get_mut("profiles")
        .and_then(Value::as_array_mut)
        .ok_or("karabiner.json missing 'profiles' array")?;

    upsert(profiles, SYSTEM_DEFAULT, json!([]));

    let files = profile_files(karaconf_dir)?;
    let count = files.len();
    for (name, path) in files {
        if name == SYSTEM_DEFAULT {
            return Err(format!("profile name '{name}' is reserved").into());
        }
        let toml_str = fs::read_to_string(&path)?;
        let profile: Profile =
            toml::from_str(&toml_str).map_err(|e| format!("parsing {}: {e}", path.display()))?;
        let rules = converter::convert(&profile)
            .map_err(|e| format!("compiling {}: {e}", path.display()))?;
        upsert(profiles, &name, serde_json::to_value(rules)?);
    }

    write_json_atomic(karabiner_json, &karabiner)?;
    println!("Synced {count} profile(s) to {}", karabiner_json.display());
    Ok(())
}

/// Switch Karabiner profiles
pub fn switch(name: &str) -> Result<(), Box<dyn Error>> {
    let status = Command::new(KARABINER_CLI)
        .args(["--select-profile", name])
        .status()?;
    if !status.success() {
        return Err(format!("karabiner_cli exited with {status}").into());
    }
    Ok(())
}

/// Reset to OS system/default
pub fn reset() -> Result<(), Box<dyn Error>> {
    switch(SYSTEM_DEFAULT)
}

/// Print switchable karaconf profile names
pub fn list(karaconf_dir: &Path) -> Result<(), Box<dyn Error>> {
    println!("{SYSTEM_DEFAULT}");
    for (name, _) in profile_files(karaconf_dir)? {
        println!("{name}");
    }
    Ok(())
}

/// Sorted `(name, path)` for every `*.toml` profile
fn profile_files(dir: &Path) -> Result<Vec<(String, PathBuf)>, Box<dyn Error>> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut out = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("invalid profile filename: {}", path.display()))?
            .to_owned();
        out.push((name, path));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn read_json(path: &Path) -> Result<Value, Box<dyn Error>> {
    let s = fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    Ok(serde_json::from_str(&s)?)
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), Box<dyn Error>> {
    let parent = path.parent().ok_or("output path has no parent")?;
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("invalid output filename")?;
    let tmp = parent.join(format!(".{filename}.tmp"));
    fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Replace only `complex_modifications.rules` on the named profile, or push
/// a fresh minimal profile if none matches. Other fields preserved.
fn upsert(profiles: &mut Vec<Value>, name: &str, rules: Value) {
    let existing = profiles
        .iter()
        .position(|p| p.get("name").and_then(Value::as_str) == Some(name));

    let Some(i) = existing else {
        profiles.push(json!({
            "name": name,
            "selected": false,
            "complex_modifications": { "rules": rules },
        }));
        return;
    };

    let profile = profiles[i]
        .as_object_mut()
        .expect("profile entry must be an object");
    let cm = profile
        .entry("complex_modifications")
        .or_insert_with(|| json!({}));
    let cm = cm
        .as_object_mut()
        .expect("complex_modifications must be an object");
    cm.insert("rules".into(), rules);
}
