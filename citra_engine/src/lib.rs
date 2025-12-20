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
#![feature(ptr_as_ref_unchecked)]

use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use crate::{
    asset_provider::{AssetProvider, AssetType},
    audio_manager::AudioManager,
    error::CitraError,
    graphics_manager::GraphicsManager,
    input::{InputManager, InputState},
    scene::{
        GameObject, Scene, SceneVersion,
        material::{Shader, ShaderVersion},
    },
    script::CitraBehaviour,
};

pub mod asset_provider;
pub mod audio_manager;
pub mod error;
pub mod graphics_manager;
pub mod input;
pub mod logger;
pub mod message;
pub mod scene;
pub mod script;
pub mod util;

thread_local! {
    pub static SCRIPTS: RefCell<Vec<Rc<RefCell<dyn CitraBehaviour>>>> = RefCell::new(vec![]);
}

pub struct EngineContext {
    pub run_start: bool,
    pub destroy_ran: bool,
    pub delta_time: std::time::Duration,
    pub tick_start: std::time::Instant,
    pub scene: Rc<RefCell<Scene>>,
    pub top_camera: Option<Rc<RefCell<GameObject>>>,
    pub bottom_camera: Option<Rc<RefCell<GameObject>>>,
    pub next_scene: Option<String>,
    pub asset_provider: Rc<RefCell<dyn AssetProvider>>,
    pub audio_manager: Rc<RefCell<dyn AudioManager>>,
    pub input_state: InputState,
    pub should_exit: bool,
}

pub struct Engine {
    context: Rc<RefCell<EngineContext>>,
    graphics_manager: Box<dyn GraphicsManager>,
    input_manager: Rc<RefCell<dyn InputManager>>,
}
impl Engine {
    pub fn new(
        asset_provider: Rc<RefCell<dyn AssetProvider>>,
        audio_manager: Rc<RefCell<dyn AudioManager>>,
        graphics_manager: Box<dyn GraphicsManager>,
        input_manager: Rc<RefCell<dyn InputManager>>,
    ) -> Self {
        Self {
            context: Rc::new(RefCell::new(EngineContext {
                run_start: true,
                destroy_ran: false,
                delta_time: std::time::Duration::from_millis(0),
                tick_start: std::time::Instant::now(),
                scene: Rc::new(RefCell::new(Scene::default())),
                top_camera: None,
                bottom_camera: None,
                next_scene: Some("/scenes/default".to_string()),
                asset_provider,
                audio_manager,
                input_state: InputState::ZERO,
                should_exit: false,
            })),
            graphics_manager,
            input_manager,
        }
    }
    fn load_scene(
        &mut self,
        next_scene: String,
        mut ctx: RefMut<EngineContext>,
    ) -> Result<(), CitraError> {
        ctx.top_camera = None;
        ctx.bottom_camera = None;
        let asset_provider = ctx.asset_provider.borrow();
        // TODO: put audio engine into scene loading mode
        let mut slice = Vec::from(asset_provider.read_file_to_string(
            asset_provider.get_asset_location(asset_provider::AssetType::Scene, next_scene),
        )?);
        let scene_version: SceneVersion = simd_json::from_slice(&mut slice)
            .map_err(|e| CitraError::SceneLoadingError(e.to_string()))?;
        let mut scene = Scene::try_from(scene_version)?;
        for light in scene.light_env.lights.iter_mut().flatten() {
            light.build_ref(scene.root.clone())?;
        }
        for model in scene.models.clone() {
            self.graphics_manager.load_model(&model[..])?;
        }
        for material in scene.materials.iter() {
            let mut shader_data = Vec::from(
                asset_provider.read_file_to_string(
                    asset_provider
                        .get_asset_location(AssetType::ShaderData, material.shader_file.clone()),
                )?,
            );
            let shader_version: ShaderVersion = simd_json::from_slice(&mut shader_data)
                .map_err(|e| CitraError::SceneLoadingError(e.to_string()))?;
            let mut shader = Shader::try_from(shader_version)?;
            shader.name = material.shader_file.clone();
            material.verify_inputs(&shader)?;
            self.graphics_manager
                .load_material(material.clone(), shader)?;
        }
        self.graphics_manager
            .prepare_light_luts(&scene.light_env.light_luts)?;
        self.graphics_manager
            .set_light_material(&scene.light_env.light_material)?;
        let old = ctx.scene.replace(scene);
        drop(old);
        drop(asset_provider);
        ctx.next_scene = None;
        let scene = ctx.scene.clone();
        ctx.top_camera = if let Some(path) = &scene.borrow().starting_top_camera_path {
            Some(scene.borrow().fetch(path.clone())?)
        } else {
            None
        };
        ctx.bottom_camera = if let Some(path) = &scene.borrow().starting_bottom_camera_path {
            Some(scene.borrow().fetch(path.clone())?)
        } else {
            None
        };
        ctx.run_start = true;
        ctx.destroy_ran = false;
        // TODO: put audio engine back into audio mode
        Ok(())
    }
    pub fn register_scripts(scripts: Vec<Rc<RefCell<dyn CitraBehaviour>>>) {
        for script in scripts {
            Self::register_script(script.clone());
        }
    }
    pub fn register_script(script: Rc<RefCell<dyn CitraBehaviour>>) {
        SCRIPTS.with_borrow_mut(|scripts| {
            scripts.push(script);
        });
    }
    pub fn get_scripts() -> Vec<Rc<RefCell<dyn CitraBehaviour>>> {
        SCRIPTS.with_borrow(|scripts| scripts.clone())
    }
    pub fn run(&mut self) {
        while self.input_manager.borrow().should_continue() && !self.context.borrow().should_exit {
            self.context.borrow_mut().input_state = self.input_manager.borrow_mut().scan();

            let ctx = self.context.clone();
            let mut ctx = ctx.borrow_mut();
            if ctx.destroy_ran
                && let Some(next_scene) = ctx.next_scene.clone()
            {
                self.load_scene(next_scene, ctx)
                    .expect("Error while loading scene");
                continue;
            }
            let now = std::time::Instant::now();
            ctx.delta_time = now - ctx.tick_start;
            ctx.tick_start = now;
            let scene = ctx.scene.clone();
            drop(ctx);
            let root = scene.borrow().root.clone();
            drop(scene);
            GameObject::tick_obj(root, self.context.clone())
                .expect("Error while ticking scene graph");

            let mut ctx = self.context.borrow_mut();
            ctx.run_start = false;
            if ctx.next_scene.is_some() {
                ctx.destroy_ran = true;
            }
            let top_camera = ctx.top_camera.clone();
            let bottom_camera = ctx.bottom_camera.clone();
            self.graphics_manager
                .update(ctx.scene.clone(), top_camera, bottom_camera);

            self.graphics_manager.wait_for_vblank();
        }
    }
}

pub mod prelude {
    pub use crate::scene::*;
}
