use std::{cell::RefCell, rc::Rc};

use citra_engine::script::{CitraBehaviour, ScriptLibrary};

use crate::{play_music::PlayMusic, start_to_exit::StartToExit};

mod play_music;
mod start_to_exit;

#[derive(Debug, Clone, Copy, Default)]
pub struct Scripts {}
impl ScriptLibrary for Scripts {
    fn get_scripts() -> Vec<Rc<RefCell<dyn CitraBehaviour>>> {
        vec![
            Rc::new(RefCell::new(StartToExit {})),
            Rc::new(RefCell::new(PlayMusic {})),
        ]
    }
}
