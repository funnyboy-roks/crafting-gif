use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    /// Output path for the GIF
    #[arg(short, long, default_value = "out.gif")]
    pub out: PathBuf,
    /// Use the dark crafting table theme from Vanilla Tweaks
    #[arg(short, long)]
    pub dark: bool,
    /// Path from which to source the recipe
    #[arg(default_value = "recipe.toml")]
    pub recipe: PathBuf,
}
