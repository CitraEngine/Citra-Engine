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

use serde::{Deserialize, Serialize};

use crate::{error::CitraError, logger::Logger};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum AssetType {
    Script,
    Texture,
    Model,
    Shader,
    ShaderData,
    Music,
    Sfx,
    Scene,
    Logfile,
    Generic,
}

pub trait AssetProvider {
    /* V Load stuff into common space V */
    // Doesn't check if the asset exists, just formats it into a system path
    fn get_asset_location(&self, asset_type: AssetType, path: String) -> String;
    // parameter path is target path, not generic asset path
    fn read_file_to_string(&self, path: String) -> Result<String, CitraError>;
    fn get_logger(&mut self, target_file: String) -> Result<Rc<RefCell<dyn Logger>>, CitraError>;
}
