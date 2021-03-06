#[cfg(feature = "cli")]
mod cli;
pub mod coloring;
#[cfg(feature = "editor")]
mod editor;
pub mod planet;
mod terrain;
mod types;

pub use types::*;

fn main() -> anyhow::Result<()> {
    cli::run()
}
