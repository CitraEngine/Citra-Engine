use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use citra_engine::{
    EngineContext,
    asset_provider::AssetType,
    error::CitraError,
    scene::GameObject,
    script::{CitraBehaviour, ScriptArg},
};

pub struct PlayMusic {}
impl CitraBehaviour for PlayMusic {
    fn get_name(&self) -> String {
        "PlayMusic".to_owned()
    }
    fn on_enable(
        &self,
        _obj: Rc<RefCell<GameObject>>,
        ctx: Rc<RefCell<EngineContext>>,
        args: &mut HashMap<String, ScriptArg>,
    ) -> Result<(), CitraError> {
        // todo!: make this a proc macro
        let music_file = {
            let arg = args
                .get(&"file".to_owned())
                .ok_or(CitraError::MissingScriptArg(
                    self.get_name(),
                    "file".to_owned(),
                ))?;
            if let ScriptArg::FilePath(path) = arg {
                Ok(path)
            } else {
                Err(CitraError::InvalidScriptArgType(
                    self.get_name(),
                    "FilePath".to_owned(),
                    format!("{:?}", arg),
                ))
            }?
        };
        let ctx = ctx.borrow();
        ctx.audio_manager.borrow_mut().bgm_play(
            Duration::from_secs(0),
            1.0,
            ctx.asset_provider
                .borrow()
                .get_asset_location(AssetType::Music, music_file.clone()),
        )?;
        Ok(())
    }
    fn on_disable(
        &self,
        _obj: Rc<RefCell<GameObject>>,
        ctx: Rc<RefCell<EngineContext>>,
        _args: &mut HashMap<String, ScriptArg>,
    ) -> Result<(), CitraError> {
        ctx.borrow()
            .audio_manager
            .borrow_mut()
            .bgm_stop(Duration::from_secs(0));
        Ok(())
    }
}
