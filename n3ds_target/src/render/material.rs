use citra_engine::{asset_provider::AssetType, scene::material};
use citro3d::{Error, math::Matrix4, render::RenderPass};

use crate::{
    asset_provider::get_asset_location,
    render::{bank::Bank, shader::Shader, texture::Texture},
};

pub enum ShaderInput {
    Texture2D { idx: usize },
}

pub struct Material {
    pub shader: Option<usize>,
    pub inputs: Vec<ShaderInput>,
}
impl Material {}

pub struct MaterialBank {
    error_shader: Shader,
    shaders: Vec<Shader>,
    reserved_shaders: Vec<Shader>,
    textures: Vec<Texture>,
    reserved_textures: Vec<Texture>,
    materials: Vec<Material>,
    reserved_materials: Vec<Material>,
}
impl MaterialBank {
    pub fn new() -> Result<Self, Error> {
        let error_shader = Shader::load_missing_shader()?;
        Ok(Self {
            error_shader,
            shaders: vec![],
            reserved_shaders: vec![],
            textures: vec![],
            reserved_textures: vec![],
            materials: vec![],
            reserved_materials: vec![],
        })
    }
    pub fn bind<'pass>(
        &'pass self,
        idx: usize,
        pass: &mut RenderPass<'pass>,
        projection: &Matrix4,
        camera_view: &glam::Mat4,
        model_view: &glam::Mat4,
    ) {
        let material = &self.materials[idx];
        let shader = if let Some(shader_idx) = material.shader {
            &self.shaders[shader_idx]
        } else {
            &self.error_shader
        };
        shader.bind(pass, projection, camera_view, model_view);
        let mut tex_bind = 0;
        for input in material.inputs.iter() {
            match input {
                ShaderInput::Texture2D { idx } => {
                    if tex_bind >= 4 {
                        panic!("Too many textures! Max: 4");
                    }
                    self.textures[*idx].bind(pass, tex_bind);
                    tex_bind += 1;
                }
            }
        }
    }
}
impl Bank for MaterialBank {
    type InputType = (material::Material, material::Shader);
    fn load(&mut self, (material, shader): Self::InputType) {
        let shader_path = get_asset_location(AssetType::Shader, shader.vertex_path.clone());
        // first pass: check shader exists
        let mut shader_idx =
            self.shaders
                .iter()
                .enumerate()
                .find_map(|(idx, shader)| -> Option<usize> {
                    if shader.get_name() == shader_path {
                        Some(idx)
                    } else {
                        None
                    }
                });
        // second pass: try to load shader
        if shader_idx.is_none()
            && let Ok(shader) = Shader::load_from_cfg(shader)
        {
            self.shaders.push(shader);
            shader_idx = Some(self.shaders.len() - 1);
        }
        // by this point if shader_idx is still none then missing shader will be used
        let mut inputs = vec![];
        if shader_idx.is_some() {
            for input in material.inputs {
                match input {
                    material::ShaderInput::Texture2D(path) => {
                        let mut texture_idx = self
                            .textures
                            .iter()
                            .enumerate()
                            .find_map(|(idx, t)| if t.name == path { Some(idx) } else { None });
                        if texture_idx.is_none()
                            && let Ok(texture) = Texture::load_from_file(path)
                        {
                            self.textures.push(texture);
                            texture_idx = Some(self.textures.len() - 1);
                        } else {
                            // TODO!: log warning
                            shader_idx = None;
                            inputs = vec![];
                            break;
                        }
                        // by this point, texture_idx is a valid idx
                        inputs.push(ShaderInput::Texture2D {
                            idx: texture_idx.unwrap(),
                        });
                    }
                    _ => todo!(),
                }
            }
        }
        self.materials.push(Material {
            shader: shader_idx,
            inputs,
        });
    }
    fn load_to_reserve(&mut self, input: Self::InputType) {
        todo!()
    }
    fn unload_all(&mut self) {
        self.shaders = vec![];
        self.materials = vec![];
    }
}
