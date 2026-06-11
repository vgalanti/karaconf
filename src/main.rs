//! CLI entry point

mod config;
mod converter;
mod karabiner;
mod keys;
mod layouts;
mod sync;

use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "karaconf",
    version,
    about = "Manage Karabiner-Elements profiles from TOML"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Sync,                    // Sync TOML files in ~/.config/karaconf/ to Karabiner profiles
    Switch { name: String }, // Switch the active Karabiner profile
    Reset,                   // Switch to the karaconf-managed system/default
    List,                    // List karaconf profile names
}

fn config_root() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".config")
}

fn karaconf_dir() -> PathBuf {
    config_root().join("karaconf")
}

fn karabiner_json() -> PathBuf {
    config_root().join("karabiner/karabiner.json")
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    match Cli::parse().command {
        Command::Sync => sync::sync(&karaconf_dir(), &karabiner_json()),
        Command::Switch { name } => sync::switch(&name),
        Command::Reset => sync::reset(),
        Command::List => sync::list(&karaconf_dir()),
    }
}
