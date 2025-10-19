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

use std::{cell::RefCell, collections::HashMap, rc::{Rc, Weak}};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::{error::CitraError, scene::{node_path::NodePath, GameObject}, util::visitors::StringVisitor, Engine, EngineContext};

pub trait CitraBehaviour {
    fn get_name(&self) -> String {"".to_string()}
    fn on_load(&self) -> Result<(), CitraError> {Ok(())}
    fn on_start(&self) -> Result<(), CitraError> {Ok(())}
    fn on_enable(&self) -> Result<(), CitraError> {Ok(())}
    fn on_tick(&self, obj: Rc<RefCell<GameObject>>, ctx: Rc<RefCell<EngineContext>>, args: &mut HashMap<String, ScriptArg>) -> Result<(), CitraError> {Ok(())}
    fn on_disable(&self) -> Result<(), CitraError> {Ok(())}
    fn on_destroy(&self) -> Result<(), CitraError> {Ok(())}
}
impl PartialEq for dyn CitraBehaviour {
    fn eq(&self, other: &Self) -> bool {
        self.get_name() == other.get_name()
    }
}
impl std::fmt::Debug for dyn CitraBehaviour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Citra Behaviour: {}", self.get_name())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScriptVariant {
    #[serde(serialize_with = "serialize_citra_behaviour_rc")]
    #[serde(deserialize_with = "deserialize_citra_behaviour_rc")]
    script: Rc<RefCell<dyn CitraBehaviour>>,
    args: HashMap<String, ScriptArg>
}
impl ScriptVariant {
    pub fn get_script_args(&self) -> HashMap<String, ScriptArg> {
        self.args.clone()
    }
    pub fn get_name(&self) -> String {self.script.borrow().get_name()}
    pub fn on_load(&self) -> Result<(), CitraError> {self.script.borrow().on_load()}
    pub fn on_start(&self) -> Result<(), CitraError> {self.script.borrow().on_start()}
    pub fn on_enable(&self) -> Result<(), CitraError> {self.script.borrow().on_enable()}
    pub fn on_tick(&mut self, obj: Rc<RefCell<GameObject>>, ctx: Rc<RefCell<EngineContext>>) -> Result<(), CitraError> {self.script.borrow().on_tick(obj, ctx, &mut self.args)}
    pub fn on_disable(&self) -> Result<(), CitraError> {self.script.borrow().on_disable()}
    pub fn on_destroy(&self) -> Result<(), CitraError> {self.script.borrow().on_destroy()}

    pub fn new(script: Rc<RefCell<dyn CitraBehaviour>>, args: HashMap<String, ScriptArg>) -> ScriptVariant {
        Self {
            script,
            args
        }
    }
}

fn serialize_citra_behaviour_rc<S>(behaviour: &Rc<RefCell<dyn CitraBehaviour>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    serializer.serialize_str(&behaviour.borrow().get_name()[..])
}
fn deserialize_citra_behaviour_rc<'de, D>(deserializer: D) -> Result<Rc<RefCell<dyn CitraBehaviour>>, D::Error>
    where D: Deserializer<'de> {
    let name = deserializer.deserialize_str(StringVisitor {})?;
    Engine::get_scripts().into_iter().find(|c| c.borrow().get_name() == name).ok_or_else(|| de::Error::custom(format!("There is no script named '{}'", name)))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ScriptArg {
    Bool(bool),
    Int(u32),
    Float(u32),
    Vec3(glam::Vec3),
    String(String),
    FilePath(String),
    NodePath(NodePath),
    #[serde(skip)] 
    NodeReference(Weak<RefCell<GameObject>>)
}