use std::{cell::RefCell, collections::HashMap, rc::Rc};

use citra_engine::{
    EngineContext,
    error::CitraError,
    scene::GameObject,
    script::{CitraBehaviour, ScriptArg},
};

pub struct StartToExit {}
impl CitraBehaviour for StartToExit {
    fn get_name(&self) -> String {
        "StartToExit".to_string()
    }
    fn on_tick(
        &self,
        _: Rc<RefCell<GameObject>>,
        ctx: Rc<RefCell<EngineContext>>,
        _: &mut HashMap<String, ScriptArg>,
    ) -> Result<(), CitraError> {
        let mut ctx = ctx.borrow_mut();
        if ctx.input_state.start_pressed() {
            ctx.should_exit = true;
        }
        Ok(())
    }
}
