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

use std::error::Error;

#[derive(Debug, Clone)]
pub enum CitraError {
    AssetNotFound(String),
    InvalidAssetFormat(String),
    ScriptError(String),
    RenderError(String),
    InvalidNodePath(String),
    InvalidMaterialIndex(String),
    InvalidMeshIndex(String),
    InvalidScriptName(String),
    PlaceholderPleaseImplement(String),
    SceneLoadingError(String),
    SceneVersionNotSupported,
    Other(String),
}
impl PartialEq for CitraError {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
impl core::fmt::Display for CitraError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CitraError::AssetNotFound(msg) => write!(f, "Asset not found: {}", msg),
            CitraError::InvalidAssetFormat(msg) => write!(f, "Invalid asset format: {}", msg),
            CitraError::ScriptError(msg) => write!(f, "Script error: {}", msg),
            CitraError::RenderError(msg) => write!(f, "Render error: {}", msg),
            CitraError::InvalidNodePath(msg) => write!(f, "Invalid node path: {}", msg),
            CitraError::InvalidMaterialIndex(msg) => write!(f, "Invalid Material Index: {}", msg),
            CitraError::InvalidMeshIndex(msg) => write!(f, "Invalid Mesh Index: {}", msg),
            CitraError::InvalidScriptName(msg) => write!(f, "Invalid Script Name: {}", msg),
            CitraError::PlaceholderPleaseImplement(msg) => write!(f, "Placeholder Please Implement: {}", msg),
            CitraError::SceneLoadingError(msg) => write!(f, "Scene Loading Error: {}", msg),
            CitraError::SceneVersionNotSupported => write!(f, "Scene Version Not Supported"),
            CitraError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}
impl Error for CitraError {}