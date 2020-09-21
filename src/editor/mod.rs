mod window;

use crate::Edit;
use kludgine::prelude::*;

pub(crate) fn run(opts: Edit) -> ! {
    SingleWindowApplication::run(window::EditorWindow::new(opts));
}
