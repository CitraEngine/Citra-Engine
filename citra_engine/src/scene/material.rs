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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ShaderInput {
    #[serde(rename = "texture")]
    Texture2D(String),
    Float(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Material {
    pub shader: String,
    pub inputs: Vec<ShaderInput>
}