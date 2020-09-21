mod coloring;
mod editor;
mod elevation;
mod planet;
mod terrain;
mod terrain_point;
mod types;

pub use types::*;

fn main() -> anyhow::Result<()> {
    editor::run()
}
