#[cfg(feature = "cli")]
mod cli;
mod coloring;
#[cfg(feature = "editor")]
mod editor;
mod planet;
mod terrain;
mod terrain_point;
mod types;

pub use types::*;

fn main() -> anyhow::Result<()> {
    cli::run()
}
