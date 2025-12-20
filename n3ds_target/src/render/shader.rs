use std::{fmt::Debug, path::Path};

use citra_engine::{
    asset_provider::AssetType,
    scene::material::{self, TexEnvCFG, TexEnvFunc, TexEnvMode, TexEnvSource},
};
use citro3d::{Error, math::Matrix4, render::RenderPass, shader, texenv, uniform::Index};
use citro3d_macros::include_shader;

use crate::{asset_provider::get_asset_location, render::model::STRIDE};

// TODO!: find a way to set up texenvs
pub struct Shader {
    name: String,
    _vertex_data: Vec<u8>,
    _geo_data: Option<Vec<u8>>,
    _vertex_library: shader::Library, // keeping this around so the DVLE data stays alive
    _geo_library: Option<shader::Library>,
    program: shader::Program,
    uloc_projection: Index,
    uloc_camera_view: Index,
    uloc_model_view: Index,
    texenv_cfg: [Option<TexEnvCFG>; 6],
}
impl Shader {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn load_from_cfg(shader_cfg: material::Shader) -> Result<Self, Error> {
        let vertex_bytes: Vec<u8> = std::fs::read(get_asset_location(
            AssetType::Shader,
            shader_cfg.vertex_path,
        ))
        .map_err(|_| Error::FailedToInitialize)?;
        if let Some(geo_shader) = shader_cfg.geo_path {
            Self::load_from_bytes(
                shader_cfg.name,
                vertex_bytes,
                Some(Vec::from(
                    std::fs::read_to_string(get_asset_location(AssetType::Shader, geo_shader))
                        .map_err(|_| Error::FailedToInitialize)?,
                )),
                shader_cfg.env_cfg,
            )
        } else {
            Self::load_from_bytes(shader_cfg.name, vertex_bytes, None, shader_cfg.env_cfg)
        }
    }
    pub fn load_from_bytes(
        name: impl ToString,
        vertex_bytes: Vec<u8>,
        geo_bytes: Option<Vec<u8>>,
        texenv_cfg: [Option<TexEnvCFG>; 6],
    ) -> Result<Self, Error> {
        let vertex_library =
            shader::Library::from_bytes(&vertex_bytes).map_err(|_| Error::FailedToInitialize)?;
        let mut program = shader::Program::new(vertex_library.get(0).unwrap())
            .map_err(|_| Error::FailedToInitialize)?;
        let mut geo_library = None;
        if let Some(geo_bytes) = &geo_bytes {
            let lib =
                shader::Library::from_bytes(geo_bytes).map_err(|_| Error::FailedToInitialize)?;
            program
                .set_geometry_shader(lib.get(0).unwrap(), STRIDE.clone().try_into().unwrap())
                .map_err(|_| Error::FailedToInitialize)?;
            geo_library = Some(lib);
        }
        let uloc_projection = program
            .get_uniform("projection")
            .map_err(|_| Error::FailedToInitialize)?;
        let uloc_camera_view = program
            .get_uniform("cameraView")
            .map_err(|_| Error::FailedToInitialize)?;
        let uloc_model_view = program
            .get_uniform("modelView")
            .map_err(|_| Error::FailedToInitialize)?;
        Ok(Self {
            name: name.to_string(),
            _vertex_data: vertex_bytes,
            _geo_data: geo_bytes,
            _vertex_library: vertex_library,
            _geo_library: geo_library,
            program,
            uloc_projection,
            uloc_camera_view,
            uloc_model_view,
            texenv_cfg,
        })
    }
    pub fn load_missing_shader() -> Result<Self, Error> {
        Self::load_from_bytes(
            "missing",
            Vec::from(include_shader!("missing.v.pica")),
            None,
            [
                Some(TexEnvCFG {
                    src_mode: TexEnvMode::Both,
                    source0: TexEnvSource::PrimaryColor,
                    source1: None,
                    source2: None,
                    func_mode: TexEnvMode::Both,
                    func: TexEnvFunc::Add,
                }),
                None,
                None,
                None,
                None,
                None,
            ],
        )
    }
    pub fn bind<'pass>(
        &'pass self,
        pass: &mut RenderPass<'pass>,
        projection: &Matrix4,
        camera_view: &glam::Mat4,
        model_view: &glam::Mat4,
    ) {
        pass.bind_program(&self.program);
        pass.bind_vertex_uniform(self.uloc_projection, projection);
        pass.bind_vertex_uniform(self.uloc_camera_view, *camera_view);
        pass.bind_vertex_uniform(self.uloc_model_view, *model_view);

        for (i, cfg) in self.texenv_cfg.iter().enumerate() {
            if let Some(cfg) = cfg {
                let stage = texenv::Stage::new(i).unwrap();
                // TODO!: convert to c3d while loading instead of during frame to save preformance
                pass.texenv(stage)
                    .src(
                        convert_mode_to_c3d(&cfg.src_mode),
                        convert_source_to_c3d(&cfg.source0),
                        convert_some_source_to_c3d(&cfg.source1),
                        convert_some_source_to_c3d(&cfg.source2),
                    )
                    .func(
                        convert_mode_to_c3d(&cfg.func_mode),
                        convert_func_to_c3d(&cfg.func),
                    );
            }
        }
    }
}
impl Debug for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("struct Shader")
    }
}
impl PartialEq for Shader {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

fn convert_mode_to_c3d(mode: &TexEnvMode) -> texenv::Mode {
    match mode {
        TexEnvMode::Rgb => texenv::Mode::RGB,
        TexEnvMode::Alpha => texenv::Mode::ALPHA,
        TexEnvMode::Both => texenv::Mode::BOTH,
    }
}

fn convert_source_to_c3d(source: &TexEnvSource) -> texenv::Source {
    match source {
        TexEnvSource::PrimaryColor => texenv::Source::PrimaryColor,
        TexEnvSource::FragmentPrimaryColor => texenv::Source::FragmentPrimaryColor,
        TexEnvSource::FragmentSecondaryColor => texenv::Source::FragmentSecondaryColor,
        TexEnvSource::Texture0 => texenv::Source::Texture0,
        TexEnvSource::Texture1 => texenv::Source::Texture1,
        TexEnvSource::Texture2 => texenv::Source::Texture2,
        TexEnvSource::Texture3 => texenv::Source::Texture3,
        TexEnvSource::PreviousBuffer => texenv::Source::PreviousBuffer,
        TexEnvSource::Constant => texenv::Source::Constant,
        TexEnvSource::Previous => texenv::Source::Previous,
    }
}

fn convert_some_source_to_c3d(source: &Option<TexEnvSource>) -> Option<texenv::Source> {
    source.map(|source| convert_source_to_c3d(&source))
}

fn convert_func_to_c3d(func: &TexEnvFunc) -> texenv::CombineFunc {
    match func {
        TexEnvFunc::Replace => texenv::CombineFunc::Replace,
        TexEnvFunc::Modulate => texenv::CombineFunc::Modulate,
        TexEnvFunc::Add => texenv::CombineFunc::Add,
        TexEnvFunc::AddSigned => texenv::CombineFunc::AddSigned,
        TexEnvFunc::Interpolate => texenv::CombineFunc::Interpolate,
        TexEnvFunc::Subtract => texenv::CombineFunc::Subtract,
        TexEnvFunc::Dot3Rgb => texenv::CombineFunc::Dot3Rgb,
    }
}
