mod window;

use crate::cli::args::Edit;
use kludgine::prelude::*;

pub(crate) fn run(opts: Edit) -> ! {
    SingleWindowApplication::run(window::EditorWindow::new(opts));
}
