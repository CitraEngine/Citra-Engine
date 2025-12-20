use std::{mem::MaybeUninit, pin::Pin};

use citra_engine::asset_provider::AssetType;
use citro3d::{Error, render::RenderPass};
use citro3d_sys::{
    C3D_Tex, C3D_TexBind, C3D_TexCube, C3D_TexDelete, Tex3DS_TextureFree, Tex3DS_TextureImport,
};

use crate::asset_provider::get_asset_location;

pub struct Texture {
    pub name: String,
    pub texture: Pin<Box<C3D_Tex>>,
    pub texcube: Pin<Box<C3D_TexCube>>,
}
impl Texture {
    pub fn load_from_file(path: String) -> Result<Self, Error> {
        Self::load_from_bytes(
            path.clone(),
            &std::fs::read(get_asset_location(AssetType::Texture, path))
                .map_err(|_| Error::InvalidName)?,
        )
    }
    pub fn load_from_bytes(name: String, data: &[u8]) -> Result<Self, Error> {
        let mut texture = MaybeUninit::uninit();
        let mut texcube = MaybeUninit::uninit();
        let t3x = unsafe {
            Tex3DS_TextureImport(
                data.as_ptr().cast(),
                data.len(),
                texture.as_mut_ptr(),
                texcube.as_mut_ptr(),
                false,
            )
        };
        if t3x.is_null() {
            return Err(Error::InvalidMemoryLocation);
        }
        let texture = Box::pin(unsafe { texture.assume_init() });
        let texcube = Box::pin(unsafe { texcube.assume_init() });
        unsafe {
            Tex3DS_TextureFree(t3x);
        }
        Ok(Self {
            name,
            texture,
            texcube,
        })
    }
    pub fn bind<'pass>(&self, _pass: &mut RenderPass<'pass>, bind_idx: usize) {
        // MEMORY SAFETY:
        // Binding a texture is part of a render pass,
        // Hence we ask for the RenderPass object to tie this to the render pass
        unsafe {
            C3D_TexBind(
                bind_idx as i32,
                (self.texture.as_ref().get_ref() as *const C3D_Tex).cast_mut(),
            );
        }
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            C3D_TexDelete(self.texture.as_mut().get_mut());
        }
    }
}
