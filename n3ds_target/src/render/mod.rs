use std::{cell::RefCell, pin::Pin, rc::Rc};

use citra_engine::{
    error::CitraError,
    graphics_manager::GraphicsManager,
    scene::{
        self, GameObject, Scene,
        light::{LUTInput, LightLUT, LightLUTCfg, LightMaterial},
        render::RenderData,
    },
};
use citro2d_sys::{C2D_DEFAULT_MAX_OBJECTS, C2D_Fini, C2D_Flush, C2D_Init};
use citro3d::{
    color::Color,
    light::{LightEnv, LightIndex, Lut, LutId, LutInput, Material},
    math::{AspectRatio, ClipPlanes, FVec3, Matrix4, Projection, StereoDisplacement},
    render::{ClearFlags, DepthFormat, RenderPass, Target},
};
use ctru::{
    Error,
    os::current_3d_slider_state,
    prelude::Gfx,
    services::gfx::{RawFrameBuffer, Screen, TopScreen3D},
};

use crate::render::{bank::Bank, material::MaterialBank, model::ModelBank};

mod bank;
mod glbhelper;
mod material;
mod model;
mod shader;
mod texture;

const CLEAR_COLOR: u32 = 0xFF_00_00_00;

pub struct N3dsGraphicsManager<'gfx> {
    gfx: &'gfx Gfx,
    citro3d: citro3d::Instance,
    top_left_target: Target<'gfx>,
    top_right_target: Target<'gfx>,
    bottom_target: Target<'gfx>,
    material_bank: MaterialBank,
    model_bank: ModelBank,
    light_env: Pin<Box<LightEnv>>,
}
impl<'gfx> N3dsGraphicsManager<'gfx> {
    pub fn new(gfx: &'gfx Gfx, top_screen: &'gfx mut TopScreen3D<'gfx>) -> Result<Self, Error> {
        let citro3d = citro3d::Instance::new().expect("Unable to initialize Citro3D");
        if !unsafe { C2D_Init(C2D_DEFAULT_MAX_OBJECTS as usize) } {
            return Err(Error::Other("Unable to initialize Citro2D".to_string()));
        }

        // top screen will also stay with the targets
        let (mut top_left, mut top_right) = top_screen.split_mut();

        let RawFrameBuffer { width, height, .. } = top_left.raw_framebuffer();
        let top_left_target = citro3d
            .render_target(width, height, top_left, Some(DepthFormat::Depth24Stencil8))
            .expect("Couldn't initialize top_left_target");
        let RawFrameBuffer { width, height, .. } = top_right.raw_framebuffer();
        let top_right_target = citro3d
            .render_target(width, height, top_right, Some(DepthFormat::Depth24Stencil8))
            .expect("Couldn't initialize top_right_target");

        // bottom screen will also stay with gfx
        let mut bottom_screen = gfx.bottom_screen.borrow_mut();
        let RawFrameBuffer { width, height, .. } = bottom_screen.raw_framebuffer();
        let bottom_target = citro3d
            .render_target(
                width,
                height,
                bottom_screen,
                Some(DepthFormat::Depth24Stencil8),
            )
            .expect("Couldn't initialize bottom_target");

        let material_bank = MaterialBank::new().expect("Unable to initialize material bank");
        let model_bank = ModelBank::new().expect("Unable to initialize model bank");

        let mut light_env = LightEnv::new_pinned();

        // create lights for later
        for _ in 0..8 {
            let idx = light_env.as_mut().create_light().unwrap();
            light_env
                .as_mut()
                .light_mut(idx)
                .unwrap()
                .set_enabled(false);
        }

        Ok(Self {
            gfx,
            citro3d,
            top_left_target,
            top_right_target,
            bottom_target,
            material_bank,
            model_bank,
            light_env,
        })
    }
}
impl<'gfx> GraphicsManager for N3dsGraphicsManager<'gfx> {
    fn load_material(
        &mut self,
        material: scene::material::Material,
        shader: scene::material::Shader,
    ) -> Result<(), CitraError> {
        self.material_bank.load((material, shader));
        Ok(())
    }
    fn load_model(&mut self, model: &str) -> Result<(), CitraError> {
        self.model_bank.load(model.to_owned());
        Ok(())
    }
    fn set_light_material(&mut self, mat: &scene::light::LightMaterial) -> Result<(), CitraError> {
        self.light_env
            .as_mut()
            .set_material(convert_mat_to_c3d(mat));
        Ok(())
    }
    fn prepare_light_luts(&mut self, cfgs: &scene::light::LightLUTCfgs) -> Result<(), CitraError> {
        if let Some(cfg) = &cfgs.diffuse0 {
            let lut = create_lut(cfg);
            self.light_env.as_mut().connect_lut(
                LutId::D0,
                convert_lut_input_to_c3d(&cfg.input),
                lut,
            );
        }
        if let Some(cfg) = &cfgs.diffuse1 {
            let lut = create_lut(cfg);
            self.light_env.as_mut().connect_lut(
                LutId::D1,
                convert_lut_input_to_c3d(&cfg.input),
                lut,
            );
        }
        if let Some(cfg) = &cfgs.spotlight {
            let lut = create_lut(cfg);
            self.light_env.as_mut().connect_lut(
                LutId::Spotlight,
                convert_lut_input_to_c3d(&cfg.input),
                lut,
            );
        }
        if let Some(cfg) = &cfgs.fresnel {
            let lut = create_lut(cfg);
            self.light_env.as_mut().connect_lut(
                LutId::Fresnel,
                convert_lut_input_to_c3d(&cfg.input),
                lut,
            );
        }
        if let Some(cfg) = &cfgs.reflection_red {
            let lut = create_lut(cfg);
            self.light_env.as_mut().connect_lut(
                LutId::ReflectRed,
                convert_lut_input_to_c3d(&cfg.input),
                lut,
            );
        }
        if let Some(cfg) = &cfgs.reflection_green {
            let lut = create_lut(cfg);
            self.light_env.as_mut().connect_lut(
                LutId::ReflectGreen,
                convert_lut_input_to_c3d(&cfg.input),
                lut,
            );
        }
        if let Some(cfg) = &cfgs.reflection_blue {
            let lut = create_lut(cfg);
            self.light_env.as_mut().connect_lut(
                LutId::ReflectBlue,
                convert_lut_input_to_c3d(&cfg.input),
                lut,
            );
        }
        if let Some(cfg) = &cfgs.distance_attenuation {
            let lut = create_lut(cfg);
            self.light_env.as_mut().connect_lut(
                LutId::DistanceAttenuation,
                convert_lut_input_to_c3d(&cfg.input),
                lut,
            );
        }
        Ok(())
    }
    fn unload_all_materials(&mut self) {
        self.material_bank.unload_all();
    }
    fn unload_all_models(&mut self) {
        self.model_bank.unload_all();
    }
    fn update(
        &mut self,
        scene: Rc<RefCell<Scene>>,
        top_camera: Option<Rc<RefCell<GameObject>>>,
        bottom_camera: Option<Rc<RefCell<GameObject>>>,
    ) {
        self.citro3d.render_frame_with(|mut pass| {
            fn cast_lifetime_to_closure<'pass, T>(x: T) -> T
            where
                T: Fn(&mut RenderPass<'pass>, &'pass Target<'_>, &Matrix4, &glam::Mat4),
            {
                x
            }

            let render_to = cast_lifetime_to_closure(|pass, target, projection, camera_view| {
                pass.select_render_target(target)
                    .expect("Could not set render target");

                let scn = scene.borrow();
                let root = scn.root.clone();
                drop(scn);

                // hashmap tree - stack traversal pattern
                let mut obj_queue = vec![root];
                while let Some(obj) = obj_queue.pop() {
                    let mut obj = obj.borrow_mut();

                    if obj.is_visible() {
                        match obj.render_data.clone() {
                            RenderData::Empty => (), // empty literally means don't render anything
                            RenderData::Cube { material } => {
                                let model_view = obj.get_transform();
                                self.material_bank.bind(
                                    material,
                                    pass,
                                    projection,
                                    camera_view,
                                    &model_view,
                                );
                                self.model_bank.draw_cube(pass);
                            }
                            RenderData::Camera { .. } => (), // we don't need to render anything for a camera
                            RenderData::Mesh { materials, mesh } => {
                                let model_view = obj.get_transform();
                                for (i, prim) in self.model_bank.models[mesh.model].meshes
                                    [mesh.mesh]
                                    .primitives
                                    .iter()
                                    .enumerate()
                                {
                                    self.material_bank.bind(
                                        materials[i],
                                        pass,
                                        projection,
                                        camera_view,
                                        &model_view,
                                    );
                                    prim.draw(pass);
                                }
                            }
                            _ => todo!(),
                        }

                        for (_, child) in obj
                            .children
                            .iter()
                            .collect::<Vec<(&String, &Rc<RefCell<GameObject>>)>>()
                            .iter()
                            .rev()
                        // have to do this ridiculousness to iterate a hashmap backwards that doesnt support DoubleEndedIterator, might look into if I can just not do this
                        {
                            obj_queue.push((*child).clone());
                        }
                    }
                }
                unsafe {
                    C2D_Flush();
                }
            });

            // this will also make sure that if a screen doesn't have a camera, it will display black instead of whatever was last on the buffer
            self.top_left_target.clear(ClearFlags::ALL, CLEAR_COLOR, 0);
            self.top_right_target.clear(ClearFlags::ALL, CLEAR_COLOR, 0);
            self.bottom_target.clear(ClearFlags::ALL, CLEAR_COLOR, 0);

            for (i, scn_light) in scene.borrow().light_env.lights.iter().enumerate() {
                let idx = LightIndex::new(i);
                let mut light = self.light_env.as_mut().light_mut(idx).unwrap();
                if let Some(scn_light) = scn_light {
                    light.as_mut().set_enabled(true);
                    light.as_mut().set_color(Color::new(
                        scn_light.color.x,
                        scn_light.color.y,
                        scn_light.color.z,
                    ));
                    if let Some(obj) = &scn_light.follow_transform {
                        let obj = obj.borrow();
                        light.as_mut().set_position(FVec3::from(obj.position));
                    }
                } else {
                    light.as_mut().set_enabled(false);
                }
            }

            // Tip: Citro2D does not do anything to lights or light envs, setting it here is safe
            pass.bind_light_env(Some(self.light_env.as_mut()));

            if let Some(top_camera) = top_camera {
                let iod = current_3d_slider_state();
                let mut top_camera = top_camera.borrow_mut();
                let (left, right) =
                    if let RenderData::Camera { near, far, fov, .. } = top_camera.render_data {
                        calculate_top_projections(fov, near, far, iod)
                    } else {
                        panic!("GameObject '{}' is not a camera", top_camera.name);
                    };
                let camera_view = top_camera.get_transform();
                drop(top_camera);
                render_to(&mut pass, &self.top_left_target, &left, &camera_view);
                if iod > f32::EPSILON {
                    render_to(&mut pass, &self.top_right_target, &right, &camera_view);
                }
            } else {
                // citro3d-rs wont apply clear screens until the screen is selected as a render target
                pass.select_render_target(&self.top_left_target)
                    .expect("Could not set render target");
                pass.select_render_target(&self.top_right_target)
                    .expect("Could not set render target");
            }
            if let Some(bottom_camera) = bottom_camera {
                let mut bottom_camera = bottom_camera.borrow_mut();
                let projection =
                    if let RenderData::Camera { fov, near, far, .. } = bottom_camera.render_data {
                        calculate_bottom_projections(fov, near, far)
                    } else {
                        panic!("GameObject '{}' is not a camera", bottom_camera.name);
                    };
                let camera_view = bottom_camera.get_transform();
                drop(bottom_camera);
                render_to(&mut pass, &self.bottom_target, &projection, &camera_view);
            } else {
                pass.select_render_target(&self.bottom_target)
                    .expect("Could not set render target");
            }

            pass
        });
    }
    fn wait_for_vblank(&self) {
        self.gfx.wait_for_vblank();
    }
}
impl<'gfx> Drop for N3dsGraphicsManager<'gfx> {
    fn drop(&mut self) {
        unsafe {
            C2D_Fini();
        }
    }
}

fn calculate_top_projections(fovy: f32, near: f32, far: f32, iod: f32) -> (Matrix4, Matrix4) {
    let iod = iod / 5.0;

    let clip_planes = ClipPlanes { near, far };
    let (left, right) = StereoDisplacement::new(iod, 2.0);

    Projection::perspective(fovy, AspectRatio::TopScreen, clip_planes).stereo_matrices(left, right)
}

fn calculate_bottom_projections(fovy: f32, near: f32, far: f32) -> Matrix4 {
    let clip_planes = ClipPlanes { near, far };

    Projection::perspective(fovy, AspectRatio::BottomScreen, clip_planes).into()
}

fn create_lut(cfg: &LightLUTCfg) -> Lut {
    match cfg.lut {
        LightLUT::Phong { factor, negative } => Lut::from_fn(make_phong_lut(factor), negative),
        _ => todo!(),
    }
}

fn make_phong_lut(factor: f32) -> impl FnMut(f32) -> f32 {
    move |x| x.powf(factor)
}

fn convert_lut_input_to_c3d(input: &LUTInput) -> LutInput {
    match input {
        LUTInput::NormalXHalfVec => LutInput::NormalHalf,
        LUTInput::ViewXHalfVec => LutInput::ViewHalf,
        LUTInput::NormalXView => LutInput::NormalView,
        LUTInput::LightVecXNormal => LutInput::LightNormal,
        LUTInput::NegLightVecXSpotVec => LutInput::LightSpotLight,
        LUTInput::CosOfPhi => LutInput::CosPhi,
    }
}

fn convert_mat_to_c3d(mat: &LightMaterial) -> Material {
    Material {
        ambient: Some(Color::new(mat.ambient.x, mat.ambient.y, mat.ambient.z)),
        diffuse: Some(Color::new(mat.diffuse.x, mat.diffuse.y, mat.diffuse.z)),
        specular0: Some(Color::new(
            mat.specular0.x,
            mat.specular0.y,
            mat.specular0.z,
        )),
        specular1: Some(Color::new(
            mat.specular1.x,
            mat.specular1.y,
            mat.specular1.z,
        )),
        emission: Some(Color::new(mat.emission.x, mat.emission.y, mat.emission.z)),
    }
}
