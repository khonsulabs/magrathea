mod window;

use kludgine::prelude::*;
use structopt::StructOpt;

pub fn run() -> ! {
    let opts = window::NewPlanetOptions::from_args();
    SingleWindowApplication::run(window::EditorWindow::new(opts));
}
