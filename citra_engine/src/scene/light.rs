use std::{cell::RefCell, default, rc::Rc};

use serde::{Deserialize, Serialize, Serializer};

use crate::{
    error::CitraError,
    scene::{GameObject, node_path::NodePath},
};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum LightType {
    PointLight,
    DirectionalLight,
    SpotLight,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Light {
    pub light_type: LightType,
    #[serde(skip_serializing)]
    #[serde(rename = "follow_transform")]
    starting_follow_transform: Option<NodePath>,
    pub color: glam::Vec3,
    #[serde(skip_deserializing)]
    #[serde(serialize_with = "serialize_follow_transform")]
    pub follow_transform: Option<Rc<RefCell<GameObject>>>, // The light's transform will be the same as the object referenced here
}
impl Light {
    pub fn new(
        light_type: LightType,
        color: glam::Vec3,
        follow_transform: Option<Rc<RefCell<GameObject>>>,
    ) -> Self {
        if let Some(follow_transform) = follow_transform {
            Self {
                light_type,
                starting_follow_transform: Some(
                    follow_transform.clone().borrow().build_node_path(),
                ),
                color,
                follow_transform: Some(follow_transform),
            }
        } else {
            Self {
                light_type,
                starting_follow_transform: None,
                color,
                follow_transform: None,
            }
        }
    }
    pub fn build_ref(&mut self, scene_graph: Rc<RefCell<GameObject>>) -> Result<(), CitraError> {
        let path = self.starting_follow_transform.clone();
        if let Some(path) = path {
            self.follow_transform = Some(GameObject::fetch_obj(scene_graph, path)?.clone());
        }
        Ok(())
    }
}

fn serialize_follow_transform<S>(
    follow_transform: &Option<Rc<RefCell<GameObject>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(follow_transform) = follow_transform {
        let follow_transform = follow_transform.borrow();
        let node_path = follow_transform.build_node_path();
        node_path.serialize(serializer)
    } else {
        serializer.serialize_unit()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum LightLUT {
    Phong { factor: f32, negative: bool },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub enum LUTInput {
    NormalXHalfVec,
    ViewXHalfVec,
    NormalXView,
    #[default]
    LightVecXNormal,
    NegLightVecXSpotVec,
    CosOfPhi,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LightLUTCfg {
    pub lut: LightLUT,
    pub input: LUTInput,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct LightLUTCfgs {
    pub diffuse0: Option<LightLUTCfg>,
    pub diffuse1: Option<LightLUTCfg>,
    pub spotlight: Option<LightLUTCfg>,
    pub fresnel: Option<LightLUTCfg>,
    pub reflection_red: Option<LightLUTCfg>,
    pub reflection_green: Option<LightLUTCfg>,
    pub reflection_blue: Option<LightLUTCfg>,
    pub distance_attenuation: Option<LightLUTCfg>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct LightMaterial {
    pub ambient: glam::Vec3,
    pub diffuse: glam::Vec3,
    pub specular0: glam::Vec3,
    pub specular1: glam::Vec3,
    pub emission: glam::Vec3,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightEnv {
    pub lights: [Option<Light>; 8],
    pub light_luts: LightLUTCfgs,
    pub light_material: LightMaterial,
}
impl Default for LightEnv {
    fn default() -> Self {
        Self {
            lights: [None, None, None, None, None, None, None, None],
            light_luts: Default::default(),
            light_material: Default::default(),
        }
    }
}
