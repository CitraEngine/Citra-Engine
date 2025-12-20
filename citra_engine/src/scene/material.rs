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

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::error::CitraError;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum TexEnvMode {
    Rgb,
    Alpha,
    Both,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum TexEnvSource {
    PrimaryColor,
    FragmentPrimaryColor,
    FragmentSecondaryColor,
    Texture0,
    Texture1,
    Texture2,
    Texture3,
    PreviousBuffer,
    Constant,
    Previous,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum TexEnvFunc {
    Replace,
    Modulate,
    Add,
    AddSigned,
    Interpolate,
    Subtract,
    Dot3Rgb,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TexEnvCFG {
    pub src_mode: TexEnvMode,
    pub source0: TexEnvSource,
    pub source1: Option<TexEnvSource>,
    pub source2: Option<TexEnvSource>,
    pub func_mode: TexEnvMode,
    pub func: TexEnvFunc,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Shader {
    #[serde(skip)]
    pub name: String,
    pub vertex_path: String,
    pub geo_path: Option<String>,
    pub inputs: Vec<ShaderInputDiscriminant>,
    pub env_cfg: [Option<TexEnvCFG>; 6],
}
impl TryFrom<ShaderVersion> for Shader {
    type Error = CitraError;
    fn try_from(value: ShaderVersion) -> Result<Self, Self::Error> {
        match value {
            ShaderVersion::Latest(s) => Ok(s),
            ShaderVersion::NotSupported => Err(CitraError::ShaderVersionNotSupported),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "version")]
pub enum ShaderVersion {
    #[serde(rename = "1")]
    Latest(Shader),
    #[serde(other)]
    #[serde(skip_serializing)]
    NotSupported,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ShaderInput {
    Texture2D(String),
    Float(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderInputDiscriminant {
    Texture2D,
    Float,
    Vec2,
    Vec3,
    Vec4,
}
impl Display for ShaderInputDiscriminant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Texture2D => f.write_str("Texture2D"),
            Self::Float => f.write_str("Float"),
            Self::Vec2 => f.write_str("Vec2"),
            Self::Vec3 => f.write_str("Vec3"),
            Self::Vec4 => f.write_str("Vec4"),
        }
    }
}
impl From<&ShaderInput> for ShaderInputDiscriminant {
    fn from(value: &ShaderInput) -> Self {
        match value {
            ShaderInput::Texture2D(_) => ShaderInputDiscriminant::Texture2D,
            ShaderInput::Float(_) => ShaderInputDiscriminant::Float,
            ShaderInput::Vec2(_) => ShaderInputDiscriminant::Vec2,
            ShaderInput::Vec3(_) => ShaderInputDiscriminant::Vec3,
            ShaderInput::Vec4(_) => ShaderInputDiscriminant::Vec4,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Material {
    pub shader_file: String,
    pub inputs: Vec<ShaderInput>,
}
impl Material {
    pub(crate) fn verify_inputs(&self, shader: &Shader) -> Result<(), CitraError> {
        let inputs: Vec<ShaderInputDiscriminant> = self
            .inputs
            .iter()
            .map(|i| ShaderInputDiscriminant::from(i))
            .collect();
        if inputs.len() != shader.inputs.len() {
            return Err(CitraError::IncompatibleShaderInputs {
                mat_inputs: inputs,
                shader_inputs: shader.inputs.clone(),
            });
        }
        for (i, input) in inputs.iter().enumerate() {
            if *input != shader.inputs[i] {
                return Err(CitraError::IncompatibleShaderInputs {
                    mat_inputs: inputs,
                    shader_inputs: shader.inputs.clone(),
                });
            }
        }
        Ok(())
    }
}
