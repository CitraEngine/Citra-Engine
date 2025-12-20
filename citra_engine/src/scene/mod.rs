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

use glam::{Mat4, Quat, Vec3};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, Visitor},
    ser::SerializeMap,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    EngineContext,
    error::CitraError,
    scene::{
        light::LightEnv, material::Material, mesh::Mesh, node_path::NodePath, render::RenderData,
    },
    script::ScriptVariant,
};

pub mod legacy;
pub mod light;
pub mod material;
pub mod mesh;
pub mod node_path;
pub mod render;
pub mod ui;

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameObject {
    #[serde(skip)]
    transform: Mat4,
    #[serde(skip)]
    visible: bool,
    #[serde(skip)]
    enabled_in_hierarchy: bool,
    #[serde(skip, default = "default_true")]
    is_compute_dirty: bool,
    #[serde(skip, default = "default_true")]
    is_render_dirty: bool,
    #[serde(skip)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
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
    pub fn new(
        name: String,
        enabled: bool,
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        scripts: Vec<ScriptVariant>,
        render_data: render::RenderData,
    ) -> Rc<RefCell<Self>> {
        let obj = Rc::new(RefCell::new(Self {
            transform: Mat4::IDENTITY,
            visible: true,
            enabled_in_hierarchy: true,
            is_compute_dirty: true,
            is_render_dirty: true,
            name,
            enabled,
            render_data,
            position,
            rotation,
            scale,
            scripts,
            parent: Weak::new(),
            children: HashMap::new(),
        }));
        obj
    }
    pub fn new_empty(
        name: String,
        enabled: bool,
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        scripts: Vec<ScriptVariant>,
    ) -> Rc<RefCell<Self>> {
        Self::new(
            name,
            enabled,
            position,
            rotation,
            scale,
            scripts,
            render::RenderData::Empty,
        )
    }
    pub fn new_cube(
        name: String,
        enabled: bool,
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        scripts: Vec<ScriptVariant>,
        material: usize,
    ) -> Rc<RefCell<Self>> {
        Self::new(
            name,
            enabled,
            position,
            rotation,
            scale,
            scripts,
            render::RenderData::Cube { material },
        )
    }
    pub fn new_mesh(
        name: String,
        enabled: bool,
        position: Vec3,
        rotation: Vec3,
        scale: Vec3,
        scripts: Vec<ScriptVariant>,
        materials: Vec<usize>,
        mesh: Mesh,
    ) -> Rc<RefCell<Self>> {
        Self::new(
            name,
            enabled,
            position,
            rotation,
            scale,
            scripts,
            render::RenderData::Mesh { materials, mesh },
        )
    }
    pub fn new_camera(
        name: String,
        enabled: bool,
        position: Vec3,
        rotation: Vec3,
        scripts: Vec<ScriptVariant>,
        fov: f32,
        near: f32,
        far: f32,
    ) -> Rc<RefCell<Self>> {
        Self::new(
            name,
            enabled,
            position,
            rotation,
            Vec3::ONE,
            scripts,
            render::RenderData::Camera {
                fov,
                near,
                far,
                inverse_transform: glam::Mat4::IDENTITY,
            },
        )
    }
    pub fn is_enabled_in_hierarchy(&mut self) -> bool {
        self.clean_compute();
        self.enabled_in_hierarchy
    }
    pub fn is_enabled(&mut self) -> bool {
        // don't need a clean for this one because it only refers to self
        self.enabled
    }
    pub fn enable(&mut self) {
        if !self.enabled {
            self.enabled = true;
            self.mark_dirty();
        }
    }
    pub fn disable(&mut self) {
        if self.enabled {
            self.enabled = false;
            self.mark_dirty();
        }
    }
    pub fn mark_dirty(&mut self) {
        self.is_compute_dirty = true;
        self.is_render_dirty = true;
        for child in self.children.values() {
            child.borrow_mut().mark_dirty();
        }
    }
    pub fn mark_compute_dirty(&mut self) {
        self.is_compute_dirty = true;
        for child in self.children.values() {
            child.borrow_mut().mark_compute_dirty();
        }
    }
    pub fn clean_compute(&mut self) {
        if self.is_compute_dirty {
            self.enabled_in_hierarchy = if !self.enabled {
                false
            } else if let Some(parent) = self.parent.upgrade() {
                parent.borrow_mut().is_enabled_in_hierarchy()
            } else {
                true
            };
        }
    }
    pub fn mark_render_dirty(&mut self) {
        self.is_render_dirty = true;
        for child in self.children.values() {
            child.borrow_mut().mark_render_dirty();
        }
    }
    pub fn clean_render(&mut self) {
        if self.is_render_dirty {
            self.visible = self.is_enabled_in_hierarchy();

            // no point in doing the following if you can't see it
            if self.visible {
                // build model space transform
                if let RenderData::Camera {
                    inverse_transform, ..
                } = &mut self.render_data
                {
                    self.transform = Mat4::from_rotation_translation(
                        Quat::from_euler(
                            glam::EulerRot::XYZ,
                            self.rotation.x,
                            self.rotation.y,
                            self.rotation.z,
                        ),
                        self.position,
                    );
                    *inverse_transform = self.transform.inverse();
                } else {
                    self.transform = Mat4::from_scale_rotation_translation(
                        self.scale,
                        Quat::from_euler(
                            glam::EulerRot::XYZ,
                            self.rotation.x,
                            self.rotation.y,
                            self.rotation.z,
                        ),
                        self.position,
                    );
                }

                // apply to parent transform to bring it to world space
                if let Some(parent) = &self.parent.upgrade() {
                    let parent_transform = parent.borrow_mut().get_transform();
                    self.transform = parent_transform * self.transform;
                }
            }

            self.is_render_dirty = false;
        }
    }
    pub fn is_visible(&mut self) -> bool {
        // despite render code not adding children to queue if visible is false
        // we still need to check for frame-by-frame state changes
        self.clean_render();
        self.visible
    }
    pub fn get_transform(&mut self) -> Mat4 {
        self.clean_render();
        match self.render_data {
            RenderData::Camera {
                inverse_transform, ..
            } => inverse_transform,
            _ => self.transform,
        }
    }
    pub fn add_child(&mut self, child: Rc<RefCell<Self>>) {
        let c = child.borrow();
        self.children.insert(c.name.clone(), child.clone());
    }
    pub fn tick_obj(
        obj: Rc<RefCell<Self>>,
        ctx: Rc<RefCell<EngineContext>>,
    ) -> Result<(), CitraError> {
        let ctx_ref = ctx.borrow();
        let run_start = ctx_ref.run_start;
        let next_scene_queued = ctx_ref.next_scene.is_some();
        drop(ctx_ref);
        let mut obj_mut = obj.borrow_mut();
        let enabled = obj_mut.enabled;
        let scripts = unsafe {
            std::mem::transmute::<&mut Vec<ScriptVariant>, &'static mut Vec<ScriptVariant>>(
                &mut obj_mut.scripts,
            )
        };
        drop(obj_mut);
        for script in scripts {
            if run_start {
                script.on_start(obj.clone(), ctx.clone())?;
            }
            if !next_scene_queued {
                if enabled {
                    script.on_tick(obj.clone(), ctx.clone())?;
                }
            } else {
                script.on_destroy(obj.clone(), ctx.clone())?;
            }
        }
        let mut obj_mut = obj.borrow_mut();
        let children = unsafe {
            std::mem::transmute::<
                &mut HashMap<String, Rc<RefCell<GameObject>>>,
                &'static mut HashMap<String, Rc<RefCell<GameObject>>>,
            >(&mut obj_mut.children)
        };
        drop(obj_mut);
        for child in children.values() {
            GameObject::tick_obj(child.clone(), ctx.clone())?;
        }
        Ok(())
    }
    pub fn fetch_obj(
        obj: Rc<RefCell<Self>>,
        path: NodePath,
    ) -> Result<Rc<RefCell<Self>>, CitraError> {
        if path.path.is_empty() {
            return Ok(obj);
        }
        for (name, child) in obj.borrow().children.iter() {
            if *name == path.path[0] {
                return Self::fetch_obj(child.clone(), path.remove_first());
            }
        }
        Err(CitraError::InvalidNodePath(format!(
            "Node in path not found: {}",
            path.path[0]
        )))
    }
    pub fn build_node_path(&self) -> NodePath {
        let mut out = NodePath::new();
        if let Some(parent) = self.parent.upgrade() {
            out.extend(&parent.borrow().build_node_path());
        }
        out.push(&self.name);
        out
    }
    pub fn register_all_children_for_obj(obj: Rc<RefCell<Self>>) {
        for child in obj.borrow_mut().children.values() {
            let mut child_mut = child.borrow_mut();
            child_mut.parent = Rc::downgrade(&obj);
            drop(child_mut);
            Self::register_all_children_for_obj(child.clone());
        }
    }
}
impl Clone for GameObject {
    fn clone(&self) -> Self {
        let mut out = Self {
            transform: self.transform,
            visible: self.visible,
            enabled_in_hierarchy: self.enabled_in_hierarchy,
            is_compute_dirty: self.is_compute_dirty,
            is_render_dirty: self.is_render_dirty,
            name: self.name.clone(),
            enabled: self.enabled,
            render_data: self.render_data.clone(),
            position: self.position,
            rotation: self.rotation,
            scale: self.scale,
            scripts: self.scripts.clone(),
            parent: Weak::new(),
            children: HashMap::new(),
        };
        for child in self.children.values() {
            let child = Rc::new(RefCell::new(child.borrow().clone()));
            out.add_child(child);
        }
        out
    }
}

fn serialize_children<S>(
    map: &HashMap<String, Rc<RefCell<GameObject>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut ser_map = serializer.serialize_map(Some(map.len()))?;
    for (name, child) in map.iter() {
        ser_map.serialize_entry(name, child.as_ref())?;
    }
    ser_map.end()
}
fn deserialize_children<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Rc<RefCell<GameObject>>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ChildMapVisitor {}
    impl<'de> Visitor<'de> for ChildMapVisitor {
        type Value = HashMap<String, Rc<RefCell<GameObject>>>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string key, struct GameObject value")
        }
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mut out = HashMap::new();
            while let Some(key) = map.next_key::<String>()? {
                let mut obj: GameObject = map.next_value()?;
                obj.name = key.clone();
                let obj = Rc::new(RefCell::new(obj));
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
    pub light_env: LightEnv,
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            models: vec![],
            materials: vec![],
            starting_top_camera_path: None,
            starting_bottom_camera_path: None,
            root: GameObject::new_empty(
                "root".to_string(),
                true,
                glam::Vec3::ZERO,
                glam::Vec3::ZERO,
                glam::Vec3::ZERO,
                vec![],
            ),
            light_env: Default::default(),
        }
    }
}
impl TryFrom<SceneVersion> for Scene {
    type Error = CitraError;
    fn try_from(value: SceneVersion) -> Result<Self, Self::Error> {
        match value {
            SceneVersion::Latest(scene) => Ok(scene),
            SceneVersion::NotSupported => Err(CitraError::SceneVersionNotSupported),
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
            root: Rc::new(RefCell::new(self.root.borrow().clone())),
            light_env: self.light_env.clone(),
        };
        GameObject::register_all_children_for_obj(out.root.clone());
        out
    }
}
impl Scene {
    pub fn fetch(&self, path: NodePath) -> Result<Rc<RefCell<GameObject>>, CitraError> {
        GameObject::fetch_obj(self.root.clone(), path)
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
where
    S: Serializer,
{
    let root = root.borrow();
    let mut map = serializer.serialize_map(Some(root.children.len()))?;
    for (name, child) in root.children.iter() {
        map.serialize_entry(name, child.as_ref())?;
    }
    map.end()
}
fn deserialize_scene_root<'de, D>(deserializer: D) -> Result<Rc<RefCell<GameObject>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ChildMapVisitorToVec {}
    impl<'de> Visitor<'de> for ChildMapVisitorToVec {
        type Value = Vec<Rc<RefCell<GameObject>>>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string key, struct GameObject value")
        }
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mut out = vec![];
            while let Some(key) = map.next_key()? {
                let mut obj: GameObject = map.next_value()?;
                obj.name = key;
                let obj = Rc::new(RefCell::new(obj));
                out.push(obj);
            }
            Ok(out)
        }
    }
    let out = GameObject::new_empty(
        "root".to_string(),
        true,
        Vec3::ZERO,
        Vec3::ZERO,
        Vec3::ONE,
        vec![],
    );
    let vec = deserializer.deserialize_map(ChildMapVisitorToVec {})?;
    for child in vec {
        GameObject::register_all_children_for_obj(child.clone());
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
    NotSupported,
}

mod tests {
    use std::f32::consts::PI;

    use glam::Vec3;

    use crate::scene::{
        GameObject, Scene, SceneVersion,
        light::{Light, LightType},
        material::{Material, ShaderInput},
        mesh::Mesh,
        node_path::NodePath,
    };

    #[test]
    fn scene_serialization() {
        let expected = "{\"version\":\"1\",\"models\":[\"/models/dingus_model\"],\"materials\":[{\"shader\":\"/shaders/dingus_shader\",\"inputs\":[{\"Texture2D\":\"/tex/dingus_texture\"}]}],\"top_camera\":\"/some/path/lol\",\"bottom_camera\":null,\"scene_graph\":{\"cube\":{\"render_data\":{\"Mesh\":{\"materials\":[0],\"mesh\":{\"model\":0,\"mesh\":0}}},\"position\":[0.0,0.0,-3.0],\"rotation\":[0.0,1.5707963705062866,3.1415927410125732],\"scale\":[1.0,1.0,1.0],\"scripts\":[],\"children\":{}},\"camera\":{\"render_data\":{\"Camera\":{\"fov\":0.6981316804885864,\"near\":0.009999999776482582,\"far\":1000.0}},\"position\":[0.0,0.0,-3.0],\"rotation\":[0.0,0.0,0.0],\"scale\":[1.0,1.0,1.0],\"scripts\":[],\"children\":{}}},\"lights\":[{\"light_type\":\"PointLight\",\"follow_transform\":null},null,null,null,null,null,null,null]}".to_string();
        let mut scene = Scene::default();
        scene.add_model("/models/dingus_model".to_string());
        scene.add_material(Material {
            shader_file: "/shaders/dingus_shader".to_string(),
            inputs: vec![ShaderInput::Texture2D("/tex/dingus_texture".to_string())],
        });
        scene.starting_top_camera_path = Some(NodePath::from("/some/path/lol".to_string()));
        let mut root = scene.root.borrow_mut();
        root.add_child(GameObject::new_mesh(
            "cube".to_string(),
            true,
            Vec3::new(0.0, 0.0, -3.0),
            Vec3::new(0.0, 90.0_f32.to_radians(), 180.0_f32.to_radians()),
            Vec3::new(1.0, 1.0, 1.0),
            vec![],
            vec![0],
            Mesh { model: 0, mesh: 0 },
        ));
        root.add_child(GameObject::new_camera(
            "camera".to_string(),
            true,
            Vec3::new(0.0, 0.0, -3.0),
            Vec3::new(0.0, 0.0, 0.0),
            vec![],
            40.0_f32.to_radians(),
            0.01,
            1000.0,
        ));
        drop(root);
        scene.light_env.lights[0] = Some(Light::new(
            LightType::PointLight,
            glam::Vec3::new(1.0, 1.0, 1.0),
            None,
        ));
        let generated_json = simd_json::to_string(&SceneVersion::Latest(scene)).unwrap();
        assert_eq!(expected, generated_json);
    }
}
