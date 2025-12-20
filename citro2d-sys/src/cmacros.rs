use core::ffi::c_uint;

#[inline]
pub const fn C2D_Color32(r: c_uint, g: c_uint, b: c_uint, a: c_uint) -> c_uint {
    a | (b << 8) | (g << 16) | (r << 24)
}
