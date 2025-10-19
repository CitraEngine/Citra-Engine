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

use crate::{error::CitraError, scene::material::Material};

pub enum Asset {
    Script(String),
    Texture(String),
    Model(String),
    Shader(String),
    Music(String),
    SFX(String),
    Scene(String)
}

pub trait AssetProvider {
    /* V Load stuff into common space V */
    // Doesn't check if the asset exists, just formats it into a system path
    fn get_asset_location(&self, asset: &Asset) -> String;
    // parameter path is target path, not generic asset path
    fn read_file_to_slice(&self, path: String) -> Result<&[u8], CitraError>;
    /* V Load stuff into target space V */
    // this should also load any shaders and textures that are part of the material
    fn load_material(&self, material: Material) -> Result<(), CitraError>;
    fn load_model(&self, model: &str) -> Result<(), CitraError>;
}

pub struct PlaceholderAssetProvider {}
impl AssetProvider for PlaceholderAssetProvider {
    fn get_asset_location(&self, _: &Asset) -> String {
        String::from("PLACEHOLDER ASSET PROVIDER PLEASE IMPLEMENT")
    }
    fn read_file_to_slice(&self, _: String) -> Result<&[u8], CitraError> {
        Err(CitraError::PlaceholderPleaseImplement("AssetProvider".to_string()))
    }
    fn load_material(&self, _: Material) -> Result<(), CitraError> {
        Err(CitraError::PlaceholderPleaseImplement("AssetProvider".to_string()))
    }
    fn load_model(&self, _: &str) -> Result<(), CitraError> {
        Err(CitraError::PlaceholderPleaseImplement("AssetProvider".to_string()))
    }
}