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
use glam::{Mat4, Quat, Vec3};
use serde::{de::{self, Visitor}, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer
};

use crate::{error::CitraError, scene::{material::Material, mesh::Mesh, node_path::NodePath, render::RenderData}, script::ScriptVariant, EngineContext};

pub mod legacy;
pub mod material;
pub mod mesh;
pub mod render;
pub mod ui;
pub mod node_path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameObject {
    #[serde(skip)]
    transform: Mat4,
    #[serde(skip)]
    is_dirty: bool,
    #[serde(skip)]
    this: Weak<RefCell<Self>>, // DO NOT CHANGE THIS TO RC, IT WILL CAUSE THIS EVERY OBJECT TO STAY IN MEMORY DUE TO A RECURSIVE REFERENCE
    #[serde(skip)]
    pub name: String,
    pub render_data: RenderData,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub scripts: Vec<ScriptVariant>,
    #[serde(skip)]
    pub parent: Weak<RefCell<Self>>, // This also should be weak to avoid a two step recusive reference
    #[serde(serialize_with = "serialize_children")]
    #[serde(deserialize_with = "deserialize_children")]
    pub children: HashMap<String, Rc<RefCell<Self>>>,
}
impl GameObject {
    pub fn new(name: String, position: Vec3, rotation: Vec3, scale: Vec3, scripts: Vec<ScriptVariant>, render_data: render::RenderData) -> Rc<RefCell<Self>> {
        let obj = Rc::new(RefCell::new(Self {
            transform: Mat4::IDENTITY,
            is_dirty: true,
            this: Weak::new(),
            name,
            render_data,
            position,
            rotation,
            scale,
            scripts,
            parent: Weak::new(),
            children: HashMap::new(),
        }));
        obj.borrow_mut().this = Rc::downgrade(&obj);
        obj
    }
    pub fn new_empty(name: String, position: Vec3, rotation: Vec3, scale: Vec3, scripts: Vec<ScriptVariant>) -> Rc<RefCell<Self>> {
        Self::new(name, position, rotation, scale, scripts, render::RenderData::Empty)
    }
    pub fn new_cube(name: String, position: Vec3, rotation: Vec3, scale: Vec3, scripts: Vec<ScriptVariant>, material: usize) -> Rc<RefCell<Self>> {
        Self::new(name, position, rotation, scale, scripts, render::RenderData::Cube { material })
    }
    pub fn new_mesh(name: String, position: Vec3, rotation: Vec3, scale: Vec3, scripts: Vec<ScriptVariant>, materials: Vec<usize>, mesh: Mesh) -> Rc<RefCell<Self>> {
        Self::new(name, position, rotation, scale, scripts, render::RenderData::Mesh { materials, mesh })
    }
    pub fn new_camera(name: String, position: Vec3, rotation: Vec3, scripts: Vec<ScriptVariant>, fov: f32, near: f32, far: f32) -> Rc<RefCell<Self>> {
        Self::new(name, position, rotation, Vec3::ONE, scripts, render::RenderData::Camera { fov, near, far })
    }
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
        for child in self.children.values() {
            child.borrow_mut().mark_dirty();
        }
    }
    pub fn get_transform(&mut self) -> Mat4 {
        if self.is_dirty {
            // build model space transform
            self.transform = Mat4::from_scale_rotation_translation(self.scale, Quat::from_euler(glam::EulerRot::XYZ, self.scale.x, self.scale.y, self.scale.z), self.position);
            // apply to parent transform to bring it to world space
            if let Some(parent) = &self.parent.upgrade() {
                let parent_transform = parent.borrow_mut().get_transform();
                self.transform = parent_transform * self.transform;
            }
            self.is_dirty = false;
        }
        self.transform
    }
    pub fn add_child(&mut self, child: Rc<RefCell<Self>>) {
        let mut c = child.borrow_mut();
        c.parent = self.this.clone();
        self.children.insert(c.name.clone(), child.clone());
    }
    pub fn tick(&mut self, ctx: Rc<RefCell<EngineContext>>) -> Result<(), CitraError> {
        for script in &mut self.scripts {
            script.on_tick(self.this.upgrade().unwrap(), ctx.clone())?;
        }
        for child in self.children.values() {
            child.borrow_mut().tick(ctx.clone())?;
        }
        Ok(())
    }
    pub fn fetch(&self, path: NodePath) -> Result<Rc<RefCell<Self>>, CitraError> {
        if path.path.is_empty() {
            return Ok(self.this.upgrade().unwrap());
        }
        for (name, child) in self.children.iter() {
            if *name == path.path[0] {
                return child.borrow().fetch(path.remove_first())
            }
        }
        Err(CitraError::InvalidNodePath(format!("Node in path not found: {}", path.path[0])))
    }
    pub fn register_all_children(&self) {
        for child in self.children.values() {
            let mut child = child.borrow_mut();
            child.parent = self.this.clone();
            child.register_all_children();
        }
    }
}
impl Clone for GameObject {
    fn clone(&self) -> Self {
        let mut out = Self {
            transform: self.transform,
            is_dirty: self.is_dirty,
            this: Weak::new(),
            name: self.name.clone(),
            render_data: self.render_data.clone(),
            position: self.position,
            rotation: self.rotation,
            scale: self.scale,
            scripts: self.scripts.clone(),
            parent: Weak::new(),
            children: HashMap::new()
        };
        for child in self.children.values() {
            let child = Rc::new(RefCell::new(child.borrow().clone()));
            let mut mut_child = child.borrow_mut();
            mut_child.this = Rc::downgrade(&child);
            drop(mut_child);
            out.add_child(child);
        }
        out
    }
}

fn serialize_children<S>(map: &HashMap<String, Rc<RefCell<GameObject>>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    let mut ser_map = serializer.serialize_map(Some(map.len()))?;
    for (name, child) in map.iter() {
        ser_map.serialize_entry(name, child.as_ref())?;
    }
    ser_map.end()
}
fn deserialize_children<'de, D>(deserializer: D) -> Result<HashMap<String, Rc<RefCell<GameObject>>>, D::Error>
    where D: Deserializer<'de> {
    struct ChildMapVisitor {}
    impl <'de> Visitor<'de> for ChildMapVisitor {
        type Value = HashMap<String, Rc<RefCell<GameObject>>>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string key, struct GameObject value")
        }
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>, {
            let mut out = HashMap::new();
            while let Some(key) = map.next_key::<String>()? {
                let mut obj: GameObject = map.next_value()?;
                obj.name = key.clone();
                let obj = Rc::new(RefCell::new(obj));
                obj.borrow_mut().this = Rc::downgrade(&obj);
                out.insert(key, obj);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_map(ChildMapVisitor {})
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub models: Vec<String>,
    pub materials: Vec<Material>,
    #[serde(rename = "top_camera")]
    pub starting_top_camera_path: Option<NodePath>,
    #[serde(rename = "bottom_camera")]
    pub starting_bottom_camera_path: Option<NodePath>,
    #[serde(rename = "scene_graph")]
    #[serde(serialize_with = "serialize_scene_root")]
    #[serde(deserialize_with = "deserialize_scene_root")]
    pub root: Rc<RefCell<GameObject>>,
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            models: vec![],
            materials: vec![],
            starting_top_camera_path: None,
            starting_bottom_camera_path: None,
            root: GameObject::new_empty("root".to_string(), glam::Vec3::ZERO, glam::Vec3::ZERO, glam::Vec3::ZERO, vec![])
        }
    }
}
impl TryFrom<SceneVersion> for Scene {
    type Error = CitraError;
    fn try_from(value: SceneVersion) -> Result<Self, Self::Error> {
        match value {
            SceneVersion::Latest(scene) => Ok(scene),
            SceneVersion::NotSupported => Err(CitraError::SceneVersionNotSupported)
        }
    }
}
impl Clone for Scene {
    fn clone(&self) -> Self {
        let out = Self {
            models: self.models.clone(),
            materials: self.materials.clone(),
            starting_top_camera_path: self.starting_top_camera_path.clone(),
            starting_bottom_camera_path: self.starting_bottom_camera_path.clone(),
            root: Rc::new(RefCell::new(self.root.borrow().clone()))
        };
        let mut root = out.root.borrow_mut();
        root.this = Rc::downgrade(&out.root);
        root.register_all_children();
        drop(root);
        out
    }
}
impl Scene {
    pub fn fetch(&self, path: NodePath) -> Result<Rc<RefCell<GameObject>>, CitraError> {
        self.root.borrow().fetch(path)
    }

    pub fn add_material(&mut self, material: Material) -> usize {
        self.materials.push(material);
        self.materials.len() - 1
    }

    pub fn add_model(&mut self, model: String) -> usize {
        self.models.push(model);
        self.models.len() - 1
    }
}

fn serialize_scene_root<S>(root: &Rc<RefCell<GameObject>>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    let root = root.borrow();
    let mut map = serializer.serialize_map(Some(root.children.len()))?;
    for (name, child) in root.children.iter() {
        map.serialize_entry(name, child.as_ref())?;
    }
    map.end()
}
fn deserialize_scene_root<'de, D>(deserializer: D) -> Result<Rc<RefCell<GameObject>>, D::Error>
    where D: Deserializer<'de> {
    struct ChildMapVisitorToVec {}
    impl <'de> Visitor<'de> for ChildMapVisitorToVec {
        type Value = Vec<Rc<RefCell<GameObject>>>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string key, struct GameObject value")
        }
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>, {
            let mut out = vec![];
            while let Some(key) = map.next_key()? {
                let mut obj: GameObject = map.next_value()?;
                obj.name = key;
                let obj = Rc::new(RefCell::new(obj));
                obj.borrow_mut().this = Rc::downgrade(&obj);
                out.push(obj);
            }
            Ok(out)
        }
    }
    let out = GameObject::new_empty("root".to_string(), Vec3::ZERO, Vec3::ZERO, Vec3::ONE, vec![]);
    let vec = deserializer.deserialize_map(ChildMapVisitorToVec {})?;
    for child in vec {
        child.borrow().register_all_children();
        out.borrow_mut().add_child(child);
    }
    Ok(out)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum SceneVersion {
    /* Example Legacy version
    #[serde(rename = "0")]
    #[serde(skip_serializing)]
    V0(SceneV0),
    */
    #[serde(rename = "1")]
    Latest(Scene),
    #[serde(other)]
    #[serde(skip_serializing)]
    NotSupported
}

mod tests {
    use std::f32::consts::PI;

    use glam::Vec3;

    use crate::scene::{material::{Material, ShaderInput}, mesh::Mesh, node_path::NodePath, GameObject, Scene, SceneVersion};

    #[test]
    fn scene_serialization() {
        let expected = r#"{"version":"1","models":["/models/dingus_model"],"materials":[{"shader":"/shaders/dingus_shader","inputs":[{"texture":"/tex/dingus_texture"}]}],"top_camera":"/some/path/lol","bottom_camera":null,"scene_graph":{"Thing":{"render_data":{"Mesh":{"materials":[0],"mesh":{"model":0,"mesh":0}}},"position":[1.0,2.5,3.5999999046325684],"rotation":[0.0,1.5707963705062866,3.1415927410125732],"scale":[2.0,2.0,2.0],"scripts":[],"children":{}}}}"#.to_string();
        let mut scene = Scene::default();
        scene.add_model("/models/dingus_model".to_string());
        scene.add_material(Material { shader: "/shaders/dingus_shader".to_string(), inputs: vec![ShaderInput::Texture2D("/tex/dingus_texture".to_string())] });
        scene.starting_top_camera_path = Some(NodePath::from("/some/path/lol".to_string()));
        scene.root.borrow_mut().add_child(GameObject::new_mesh("Thing".to_string(), Vec3::new(1.0, 2.5, 3.6), Vec3::new(0.0, 90.0 * (PI/180.0), 180.0 * (PI/180.0)), Vec3::new(2.0, 2.0, 2.0), vec![], vec![0], Mesh { model: 0, mesh: 0}));
        let generated_json = simd_json::to_string(&SceneVersion::Latest(scene)).unwrap();
        assert_eq!(expected, generated_json);
    }
}