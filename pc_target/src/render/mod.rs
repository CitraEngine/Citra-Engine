use std::{cell::RefCell, error::Error, rc::Rc, sync::Arc};

use citra_engine::{
    error::CitraError,
    graphics_manager::GraphicsManager,
    scene::{GameObject, Scene},
};
use wgpu::{
    Backends, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, Limits, MemoryHints, PowerPreference, PresentMode, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages, Trace,
};
use winit::window::Window;

pub struct PcGraphicsManager {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    surface_configured: bool,
    window: Arc<Window>,
}
impl PcGraphicsManager {
    pub async fn new(window: Arc<Window>) -> Result<Self, CitraError> {
        let size = window.inner_size();
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::VULKAN | Backends::DX12 | Backends::METAL | Backends::GL,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| CitraError::StructInitializationError(e.to_string()))?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| CitraError::StructInitializationError(e.to_string()))?;

        let adapter = instance
            .enumerate_adapters(Backends::all())
            .into_iter()
            .filter(|adapter| adapter.is_surface_supported(&surface))
            .next()
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: Limits::defaults(),
                memory_hints: MemoryHints::Performance,
                trace: Trace::Off,
            })
            .await
            .map_err(|e| CitraError::StructInitializationError(e.to_string()))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            surface_configured: false,
            window,
        })
    }
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.surface_configured = true;
        }
    }
}
impl GraphicsManager for PcGraphicsManager {
    fn load_material(
        &mut self,
        material: citra_engine::prelude::material::Material,
    ) -> Result<(), citra_engine::error::CitraError> {
        todo!()
    }
    fn load_model(&mut self, model: &str) -> Result<(), CitraError> {
        todo!()
    }
    fn unload_all_materials(&mut self) {
        todo!()
    }
    fn unload_all_models(&mut self) {
        todo!()
    }
    fn update(
        &mut self,
        scene: Rc<RefCell<Scene>>,
        top_camera: Option<Rc<RefCell<GameObject>>>,
        bottom_camera: Option<Rc<RefCell<GameObject>>>,
    ) {
        todo!()
    }
    fn wait_for_vblank(&self) {
        todo!()
    }
}
