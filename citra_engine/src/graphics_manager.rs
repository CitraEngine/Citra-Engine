use std::{cell::RefCell, rc::Rc};

use crate::{
    error::CitraError,
    scene::{
        GameObject, Scene,
        light::{LightLUTCfgs, LightMaterial},
        material::{Material, Shader},
    },
};

pub trait GraphicsManager {
    fn load_material(&mut self, material: Material, shader: Shader) -> Result<(), CitraError>;
    fn load_model(&mut self, model: &str) -> Result<(), CitraError>;
    fn prepare_light_luts(&mut self, cfgs: &LightLUTCfgs) -> Result<(), CitraError>;
    fn set_light_material(&mut self, mat: &LightMaterial) -> Result<(), CitraError>;
    fn unload_all_materials(&mut self);
    fn unload_all_models(&mut self);
    fn update(
        &mut self,
        scene: Rc<RefCell<Scene>>,
        top_camera: Option<Rc<RefCell<GameObject>>>,
        bottom_camera: Option<Rc<RefCell<GameObject>>>,
    );
    fn wait_for_vblank(&self);
}
