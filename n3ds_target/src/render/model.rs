use std::{
    cell::RefCell,
    ffi::{c_float, c_int},
    fs::File,
    mem::MaybeUninit,
    rc::{Rc, Weak},
};

use citra_engine::{asset_provider::AssetType, error::CitraError};
use citro3d::{attrib, buffer, render::RenderPass};
use citro3d_sys::{
    C3D_BufInfo, C3D_DrawArrays, C3D_DrawElements, C3D_SetBufInfo, C3D_UNSIGNED_BYTE,
    C3D_UNSIGNED_SHORT,
};
use ctru::linear::LinearAllocator;
use ctru_sys::GPU_TRIANGLES;
use gltf_json::{accessor::ComponentType, mesh::Semantic, validation::Checked};

use crate::{
    asset_provider::get_asset_location,
    render::{
        bank::Bank,
        glbhelper::{
            SUPPORTED_ASSET_VERSION, get_accessor_component_type, get_accessor_size,
            iterate_accessor_with_index, read_header, verbatim_cast_from_accessor,
        },
    },
};

lazy_static::lazy_static! {
    pub static ref ATTR_INFO: attrib::Info = Vertex::build_attrib_info();
    pub static ref STRIDE: isize = std::mem::size_of::<Vertex>().try_into().unwrap();
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [c_float; 3],
    pub texcoord: [c_float; 2],
    pub normal: [c_float; 3],
}
impl Vertex {
    fn build_attrib_info() -> attrib::Info {
        let mut attr_info = attrib::Info::new();

        let reg0 = attrib::Register::new(0).unwrap();
        let reg1 = attrib::Register::new(1).unwrap();
        let reg2 = attrib::Register::new(2).unwrap();

        attr_info
            .add_loader(reg0, attrib::Format::Float, 3)
            .unwrap();
        attr_info
            .add_loader(reg1, attrib::Format::Float, 2)
            .unwrap();
        attr_info
            .add_loader(reg2, attrib::Format::Float, 3)
            .unwrap();

        attr_info
    }
}

pub enum IndexData {
    U16Bit { idx_data: Vec<u16, LinearAllocator> },
    U8Bit { idx_data: Vec<u8, LinearAllocator> },
    NotPresent,
}

pub struct BufInfo {
    raw: C3D_BufInfo,
}
impl BufInfo {
    fn new() -> Self {
        Self::default()
    }
    fn add(
        &mut self,
        vbo_data: &mut Vec<Vertex, LinearAllocator>,
        attrib_info: &attrib::Info,
        self_rc: &Rc<RefCell<Self>>,
    ) -> Result<VboSlice, CitraError> {
        // SAFETY: The vbo buffer and this vbo info stay together via the Mesh and Model parent child relationship,
        // linear memory for the vbo is only dropped when Mesh is dropped
        let res = unsafe {
            citro3d_sys::BufInfo_Add(
                &mut self.raw,
                vbo_data.as_mut_ptr().cast(),
                STRIDE.clone(),
                attrib_info.attr_count(),
                0x210,
            )
        };

        // Error codes from <https://github.com/devkitPro/citro3d/blob/master/source/buffers.c#L11>
        match res {
            ..=-3 => Err(CitraError::PlatformError(format!("System Error: {}", res))),
            -2 => Err(CitraError::PlatformError(
                "Invalid Memory Location".to_owned(),
            )),
            -1 => Err(CitraError::PlatformError("Too Many Buffers".to_owned())),
            _ => Ok(VboSlice {
                index: res,
                size: vbo_data.len() as i32,
                info: Rc::downgrade(self_rc),
            }),
        }
    }
    pub fn bind<'pass>(&self, pass: &mut RenderPass<'pass>) {
        let raw: *const _ = &self.raw;
        pass.set_attr_info(&ATTR_INFO);
        unsafe {
            C3D_SetBufInfo(raw.cast_mut());
        }
    }
}
impl Default for BufInfo {
    fn default() -> Self {
        let mut info = MaybeUninit::zeroed();
        let info = unsafe {
            citro3d_sys::BufInfo_Init(info.as_mut_ptr());
            info.assume_init()
        };
        Self { raw: info }
    }
}

pub struct VboSlice {
    index: c_int,
    size: c_int,
    info: Weak<RefCell<BufInfo>>,
}

pub struct Primitive {
    pub index_data: IndexData,
    pub _vbo_data: Vec<Vertex, LinearAllocator>,
    pub vbo_slice: VboSlice,
}
impl Primitive {
    pub fn draw<'pass>(&'pass self, pass: &mut RenderPass<'pass>) {
        self.vbo_slice.info.upgrade().unwrap().borrow().bind(pass);
        match &self.index_data {
            IndexData::NotPresent => unsafe {
                C3D_DrawArrays(GPU_TRIANGLES, self.vbo_slice.index, self.vbo_slice.size);
            },
            IndexData::U8Bit { idx_data } => unsafe {
                C3D_DrawElements(
                    GPU_TRIANGLES,
                    idx_data.len() as i32,
                    C3D_UNSIGNED_BYTE as i32,
                    idx_data.as_ptr().cast(),
                );
            },
            IndexData::U16Bit { idx_data } => unsafe {
                C3D_DrawElements(
                    GPU_TRIANGLES,
                    idx_data.len() as i32,
                    C3D_UNSIGNED_SHORT as i32,
                    idx_data.as_ptr().cast(),
                );
            },
        }
    }
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
    pub vbo_info: Rc<RefCell<BufInfo>>,
}
impl Mesh {
    pub fn new() -> Self {
        Self {
            primitives: vec![],
            vbo_info: Rc::new(RefCell::new(BufInfo::new())),
        }
    }
    pub fn load_cube_mesh() -> Result<Self, CitraError> {
        let mut out = Self::new();
        let mut vbo_data = Vec::with_capacity_in(CUBE_DATA.len(), LinearAllocator);
        vbo_data.extend_from_slice(&CUBE_DATA);
        out.add_array_from_memory(vbo_data)?;
        Ok(out)
    }
    fn add_array_from_memory(
        &mut self,
        mut vbo_data: Vec<Vertex, LinearAllocator>,
    ) -> Result<(), CitraError> {
        let vbo_slice =
            self.vbo_info
                .borrow_mut()
                .add(&mut vbo_data, &ATTR_INFO, &self.vbo_info)?;
        self.primitives.push(Primitive {
            index_data: IndexData::NotPresent,
            _vbo_data: vbo_data,
            vbo_slice,
        });
        Ok(())
    }
}

pub struct Model {
    pub name: String,
    pub meshes: Vec<Mesh>,
}
impl Model {
    fn load_from_glb<P>(path: P) -> Result<Self, CitraError>
    where
        P: AsRef<str>,
    {
        let data = File::open(get_asset_location(
            AssetType::Model,
            path.as_ref().to_owned(),
        ))
        .map_err(|e| CitraError::PlatformError(e.to_string()))?;
        let (root, bin_chunk_header) = read_header(&data, path.as_ref())
            .map_err(|e| CitraError::PlatformError(e.to_string()))?;

        assert_eq!(
            root.asset.version,
            SUPPORTED_ASSET_VERSION,
            "File '{}' contains unsupported asset version, Expected: {}, Got: {}",
            path.as_ref(),
            SUPPORTED_ASSET_VERSION,
            root.asset.version
        );

        let mut out = Model {
            name: path.as_ref().to_owned(),
            meshes: vec![],
        };

        for gl_mesh in root.meshes.iter() {
            let mut mesh = Mesh {
                primitives: vec![],
                vbo_info: Rc::new(RefCell::new(BufInfo::new())),
            };
            for gl_primitive in gl_mesh.primitives.iter() {
                let position_idx = gl_primitive
                    .attributes
                    .get(&Checked::Valid(Semantic::Positions));
                let position_idx = if let Some(position_idx) = position_idx {
                    position_idx
                } else {
                    return Err(CitraError::PlatformError(
                        "Position accessor not found".to_owned(),
                    ));
                };
                let normal_idx = gl_primitive
                    .attributes
                    .get(&Checked::Valid(Semantic::Normals));
                let normal_idx = if let Some(normal_idx) = normal_idx {
                    normal_idx
                } else {
                    return Err(CitraError::PlatformError(
                        "Normal accessor not found".to_owned(),
                    ));
                };

                let texcoord_idx = if let Some(mat_idx) = &gl_primitive.material {
                    if let Some(mat) = root.materials.get(mat_idx.value()) {
                        if let Some(bmt) = &mat.pbr_metallic_roughness.base_color_texture {
                            gl_primitive
                                .attributes
                                .get(&Checked::Valid(Semantic::TexCoords(bmt.tex_coord)))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                let mut vbo_data =
                    Vec::with_capacity_in(get_accessor_size(&root, position_idx)?, LinearAllocator);

                iterate_accessor_with_index::<[f32; 3], _>(
                    &root,
                    &bin_chunk_header,
                    &data,
                    position_idx,
                    vbo_data.capacity(),
                    |data, _| {
                        vbo_data.push(Vertex {
                            position: data,
                            texcoord: [0.0; 2],
                            normal: [0.0; 3],
                        });
                    },
                )?;

                if let Some(texcoord_idx) = texcoord_idx {
                    iterate_accessor_with_index::<[f32; 2], _>(
                        &root,
                        &bin_chunk_header,
                        &data,
                        texcoord_idx,
                        vbo_data.capacity(),
                        |data, i| {
                            vbo_data[i].texcoord[0] = data[0];
                            vbo_data[i].texcoord[1] = -data[1] + 1.0;
                        },
                    )?;
                }

                iterate_accessor_with_index::<[f32; 3], _>(
                    &root,
                    &bin_chunk_header,
                    &data,
                    normal_idx,
                    vbo_data.capacity(),
                    |data, i| {
                        vbo_data[i].normal = data;
                    },
                )?;

                let index_data = if let Some(index_idx) = &gl_primitive.indices {
                    match get_accessor_component_type(&root, index_idx)? {
                        Checked::Valid(g) => match g.0 {
                            ComponentType::U8 => {
                                let idx_data = verbatim_cast_from_accessor(
                                    &root,
                                    &bin_chunk_header,
                                    &data,
                                    index_idx,
                                    get_accessor_size(&root, index_idx)?,
                                )?;
                                IndexData::U8Bit { idx_data }
                            }
                            ComponentType::U16 => {
                                let idx_data = verbatim_cast_from_accessor(
                                    &root,
                                    &bin_chunk_header,
                                    &data,
                                    index_idx,
                                    get_accessor_size(&root, index_idx)?,
                                )?;
                                IndexData::U16Bit { idx_data }
                            }
                            _ => {
                                return Err(CitraError::PlatformError(
                                    "The PICA200 gpu only supports u8 and u16 indicies".to_owned(),
                                ));
                            }
                        },
                        Checked::Invalid => {
                            return Err(CitraError::PlatformError("Invalid Checked".to_owned()));
                        }
                    }
                } else {
                    IndexData::NotPresent
                };

                let vbo_slice = mesh
                    .vbo_info
                    .borrow_mut()
                    .add(&mut vbo_data, &ATTR_INFO, &mesh.vbo_info)
                    .map_err(|_| {
                        CitraError::PlatformError("Failed to add to buffer info".to_owned())
                    })?;
                mesh.primitives.push(Primitive {
                    index_data,
                    _vbo_data: vbo_data,
                    vbo_slice,
                });
            }
            out.meshes.push(mesh);
        }

        Ok(out)
    }
}

pub struct ModelBank {
    cube_mesh: Mesh,
    pub models: Vec<Model>,
    pub reserved_models: Vec<Model>,
}
impl ModelBank {
    pub fn new() -> Result<Self, CitraError> {
        Ok(Self {
            cube_mesh: Mesh::load_cube_mesh()?,
            models: vec![],
            reserved_models: vec![],
        })
    }
    pub fn draw_cube<'pass>(&'pass self, pass: &mut RenderPass<'pass>) {
        self.cube_mesh.primitives[0].draw(pass);
    }
}
impl Bank for ModelBank {
    type InputType = String;
    fn load(&mut self, input: Self::InputType) {
        self.models.push(
            Model::load_from_glb(&input)
                .unwrap_or_else(|_| panic!("Unable to load model: {}", &input)),
        );
    }
    fn load_to_reserve(&mut self, input: Self::InputType) {
        todo!()
    }
    fn unload_all(&mut self) {
        self.models = vec![];
    }
}

const CUBE_DATA: [Vertex; 36] = [
    // First face (PZ)
    // First triangle
    Vertex {
        position: [-0.5, -0.5, 0.5],
        texcoord: [0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        texcoord: [1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    // Second triangle
    Vertex {
        position: [0.5, 0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        texcoord: [0.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        texcoord: [0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
    // Second face (MZ)
    // First triangle
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        texcoord: [1.0, 0.0],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        texcoord: [1.0, 1.0],
        normal: [0.0, 0.0, -1.0],
    },
    // Second triangle
    Vertex {
        position: [0.5, 0.5, -0.5],
        texcoord: [1.0, 1.0],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        texcoord: [0.0, 1.0],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [0.0, 0.0, -1.0],
    },
    // Third face (PX)
    // First triangle
    Vertex {
        position: [0.5, -0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        texcoord: [1.0, 0.0],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
    },
    // Second triangle
    Vertex {
        position: [0.5, 0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        texcoord: [0.0, 1.0],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [1.0, 0.0, 0.0],
    },
    // Fourth face (MX)
    // First triangle
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        texcoord: [1.0, 0.0],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
    },
    // Second triangle
    Vertex {
        position: [-0.5, 0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        texcoord: [0.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [-1.0, 0.0, 0.0],
    },
    // Fifth face (PY)
    // First triangle
    Vertex {
        position: [-0.5, 0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        texcoord: [1.0, 0.0],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
    },
    // Second triangle
    Vertex {
        position: [0.5, 0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        texcoord: [0.0, 1.0],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [0.0, 1.0, 0.0],
    },
    // Sixth face (MY)
    // First triangle
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        texcoord: [1.0, 0.0],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
    },
    // Second triangle
    Vertex {
        position: [0.5, -0.5, 0.5],
        texcoord: [1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        texcoord: [0.0, 1.0],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texcoord: [0.0, 0.0],
        normal: [0.0, -1.0, 0.0],
    },
];
