/*
Citra Engine - A Nintendo 3DS first game engine
Copyright (C) 2025  Citra Engine

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::{cell::RefCell, rc::Rc};

use crate::{asset_provider::{Asset, AssetProvider, PlaceholderAssetProvider}, audio_manager::{AudioManager, PlaceholderAudioManager}, error::CitraError, scene::{GameObject, Scene, SceneVersion}, script::CitraBehaviour};

pub mod asset_provider;
pub mod audio_manager;
pub mod error;
pub mod input;
pub mod message;
pub mod scene;
pub mod script;
pub mod util;

thread_local! {
    static ENGINE: RefCell<Engine> = RefCell::new(Engine {
        context: Rc::new(RefCell::new(EngineContext {
            delta_time: std::time::Duration::from_millis(0),
                tick_start: std::time::Instant::now(),
                scene: Rc::new(RefCell::new(Scene::default())),
                top_camera: None,
                bottom_camera: None,
                next_scene: Some("/scenes/default".to_string()),
                asset_provider: Rc::new(RefCell::new(PlaceholderAssetProvider {})),
                audio_manager: Rc::new(RefCell::new(PlaceholderAudioManager {}))
            })),
            scripts: vec![]
        });
}

pub struct EngineContext {
    pub delta_time: std::time::Duration,
    pub tick_start: std::time::Instant,
    pub scene: Rc<RefCell<Scene>>,
    pub top_camera: Option<Rc<RefCell<GameObject>>>,
    pub bottom_camera: Option<Rc<RefCell<GameObject>>>,
    pub next_scene: Option<String>,
    pub asset_provider: Rc<RefCell<dyn AssetProvider>>,
    pub audio_manager: Rc<RefCell<dyn AudioManager>>
}

pub struct Engine {
    context: Rc<RefCell<EngineContext>>,
    scripts: Vec<Rc<RefCell<dyn crate::script::CitraBehaviour>>>
}
impl Engine {
    pub fn update() {
        ENGINE.with_borrow(|engine| {
            let mut ctx = engine.context.borrow_mut();
            if let Some(next_scene) = ctx.next_scene.clone() {
                drop(ctx);
                let _ = engine.load_scene(next_scene).is_err_and(|e| panic!("{}", e));
                return;
            }
            let now = std::time::Instant::now();
            ctx.delta_time = now - ctx.tick_start;
            ctx.tick_start = now;
            let scene = ctx.scene.clone();
            drop(ctx);
            let root = scene.borrow().root.clone();
            drop(scene);
            let _ = root.borrow_mut().tick(engine.context.clone()).is_err_and(|e| panic!("{}", e));
        })
    }
    fn load_scene(&self, next_scene: String) -> Result<(), CitraError> {
        let ctx = self.context.borrow();
        let asset_provider = ctx.asset_provider.borrow();
        // TODO: put audio engine into scene loading mode
        let mut slice = Vec::from(asset_provider.read_file_to_slice(asset_provider.get_asset_location(&Asset::Scene(next_scene)))?);
        let scene_version: SceneVersion = simd_json::from_slice(&mut slice).map_err(|e| CitraError::SceneLoadingError(e.to_string()))?;
        let scene = Scene::try_from(scene_version)?;
        for model in scene.models {
            asset_provider.load_model(&model[..])?;
        }
        for material in scene.materials {
            asset_provider.load_material(material)?;
        }
        // TODO: put audio engine back into audio mode
        Ok(())
    }
    pub fn get_scripts() -> Vec<Rc<RefCell<dyn CitraBehaviour>>> {
        ENGINE.with_borrow(|engine| {
            engine.scripts.clone()
        })
    }
    pub fn register_asset_provider(asset_provider: Rc<RefCell<dyn AssetProvider>>) {
        ENGINE.with_borrow(|engine| {
            engine.context.borrow_mut().asset_provider = asset_provider;
        });
    }
    pub fn get_asset_provider() -> Rc<RefCell<dyn AssetProvider>> {
        ENGINE.with_borrow(|engine| {
            engine.context.borrow().asset_provider.clone()
        })
    }
    pub fn register_audio_manager(audio_manager: Rc<RefCell<dyn AudioManager>>) {
        ENGINE.with_borrow(|engine| {
            engine.context.borrow_mut().audio_manager = audio_manager;
        });
    }
    pub fn get_audio_manager() -> Rc<RefCell<dyn AudioManager>> {
        ENGINE.with_borrow(|engine| {
            engine.context.borrow().audio_manager.clone()
        })
    }
}

pub mod prelude {
    pub use crate::scene::*;
}