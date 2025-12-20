use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    mem::MaybeUninit,
};

use citra_engine::error::CitraError;
use ctru::linear::LinearAllocator;
use gltf_json::{Accessor, Index, Root, accessor::GenericComponentType, validation::Checked};

const MIN_SUPPORTED: u32 = 2;
const MAX_SUPPORTED: u32 = 2;
pub const SUPPORTED_ASSET_VERSION: &str = "2.0";

const GLTF_MAGIC: u32 = 0x46546C67;
const JSON_CHUNK_MAGIC: u32 = 0x4E4F534A;
const BIN_CHUNK_MAGIC: u32 = 0x004E4942;

#[repr(C, packed)]
struct GlbHeader {
    pub magic: u32,
    pub version: u32,
    pub length: u32,
}
impl GlbHeader {
    pub fn get_magic(&self) -> u32 {
        self.magic
    }
    pub fn get_version(&self) -> u32 {
        self.version
    }
    pub fn get_length(&self) -> u32 {
        self.length
    }
}
#[repr(C, packed)]
pub struct ChunkHeader {
    pub length: u32,
    pub chunk_type: u32,
    pub chunk_location: usize,
}
impl ChunkHeader {
    pub fn get_length(&self) -> u32 {
        self.length
    }
    pub fn get_type(&self) -> u32 {
        self.chunk_type
    }
    pub fn get_location(&self) -> usize {
        self.chunk_location
    }
}

pub fn read_header(
    mut file_stream: &File,
    path: impl AsRef<str>,
) -> Result<(Root, ChunkHeader), CitraError> {
    let size = file_stream
        .seek(SeekFrom::End(0))
        .map_err(|e| CitraError::PlatformError(e.to_string()))?;
    file_stream
        .seek(SeekFrom::Start(0))
        .map_err(|e| CitraError::PlatformError(e.to_string()))?;

    if size < 12 {
        return Err(CitraError::PlatformError(
            "Steam size is not big enough to read the header".to_owned(),
        ));
    }

    let mut buf: [u8; 12] = [0; 12];
    file_stream
        .read_exact(&mut buf)
        .map_err(|e| CitraError::PlatformError(e.to_string()))?;

    // there is a safe way to do this but this is faster
    let mut header: MaybeUninit<GlbHeader> = MaybeUninit::uninit();
    unsafe {
        (header.as_mut_ptr() as *mut u8).copy_from_nonoverlapping(buf.as_ptr(), 12);
    }
    let header: GlbHeader = unsafe { header.assume_init() };

    assert_eq!(
        header.get_magic(),
        GLTF_MAGIC,
        "File '{}' is not a valid GLTF file",
        path.as_ref()
    );
    assert!(
        header.get_version() >= MIN_SUPPORTED,
        "File '{}' is too old, supported version range: {} - {}",
        path.as_ref(),
        MIN_SUPPORTED,
        MAX_SUPPORTED
    );
    assert!(
        header.get_version() <= MAX_SUPPORTED,
        "File '{}' is too new, supported version range: {} - {}",
        path.as_ref(),
        MIN_SUPPORTED,
        MAX_SUPPORTED
    );

    let mut root: Option<Root> = None;
    let mut bin_chunk_header: Option<ChunkHeader> = None;

    while file_stream.stream_position().unwrap() < header.get_length() as u64 {
        let chunk = read_chunk(file_stream);
        if chunk.get_type() == JSON_CHUNK_MAGIC {
            if root.is_some() {
                return Err(CitraError::PlatformError(format!(
                    "File '{}' contains too many json chunks",
                    path.as_ref()
                )));
            }
            let mut bytes: Vec<u8> = vec![0; chunk.get_length() as usize];
            file_stream.read_exact(&mut bytes).unwrap();
            root = Some(simd_json::from_slice(&mut bytes).unwrap());
        } else if chunk.get_type() == BIN_CHUNK_MAGIC {
            if bin_chunk_header.is_some() {
                return Err(CitraError::PlatformError(format!(
                    "File '{}' contains too many bin chunks",
                    path.as_ref()
                )));
            }
            bin_chunk_header = Some(chunk);
        }
    }

    if let Some(root) = root
        && let Some(bin_chunk_header) = bin_chunk_header
    {
        Ok((root, bin_chunk_header))
    } else {
        Err(CitraError::PlatformError(format!(
            "File '{}' does not contain a chunk header",
            path.as_ref()
        )))
    }
}

// WARNING: THIS FUNCTION ASSUMES THE FILE POINTER IS AT THE START OF A VALID CHUNK HEADER
// if bin chunk then seek head goes to next chunk header
// if json chunk then seek head goes to start of json data
pub fn read_chunk(mut file_stream: &File) -> ChunkHeader {
    let mut buf: [u8; 8] = [0; 8];
    file_stream.read_exact(&mut buf).unwrap();

    let mut header: MaybeUninit<ChunkHeader> = MaybeUninit::uninit();
    unsafe {
        (header.as_mut_ptr() as *mut u8).copy_from_nonoverlapping(buf.as_ptr(), 8);
    }
    let mut header = unsafe { header.assume_init() };
    header.chunk_location = file_stream.stream_position().unwrap().try_into().unwrap();
    match header.get_type() {
        BIN_CHUNK_MAGIC => {
            file_stream
                .seek(SeekFrom::Current(header.get_length().into()))
                .unwrap();
            header
        }
        JSON_CHUNK_MAGIC => header,
        n => panic!("Invalid chunk magic: {}", n),
    }
}

pub fn get_accessor_size(root: &Root, accessor_idx: &Index<Accessor>) -> Result<usize, CitraError> {
    let accessor = root
        .accessors
        .get(accessor_idx.value())
        .ok_or(CitraError::PlatformError(
            "Could not find accessor".to_owned(),
        ))?;
    accessor.count.0.try_into().map_err(|_| {
        CitraError::PlatformError("Accessor count could not be converted to usize".to_owned())
    })
}

pub fn iterate_accessor_with_index<T, F>(
    root: &Root,
    bin_chunk_header: &ChunkHeader,
    mut file: &File,
    accessor_idx: &Index<Accessor>,
    accessor_count: usize,
    mut func: F,
) -> Result<(), CitraError>
where
    T: Sized,
    F: FnMut(T, usize),
{
    let accessor = root
        .accessors
        .get(accessor_idx.value())
        .ok_or(CitraError::PlatformError(
            "Could not find accessor".to_owned(),
        ))?;
    let buffer_view_idx = accessor.buffer_view.ok_or(CitraError::PlatformError(
        "Accessor does not contain buffer view idx".to_owned(),
    ))?;
    let buffer_view =
        root.buffer_views
            .get(buffer_view_idx.value())
            .ok_or(CitraError::PlatformError(
                "Buffer View index does not exist".to_owned(),
            ))?;
    let byte_offset: usize = buffer_view
        .byte_offset
        .ok_or(CitraError::PlatformError(
            "Buffer View does not have an offset".to_owned(),
        ))?
        .0
        .try_into()
        .map_err(|_| CitraError::PlatformError("Could not convert u64 to usize".to_owned()))?;

    file.seek(SeekFrom::Start(
        (bin_chunk_header.get_location() + byte_offset) as u64,
    ))
    .map_err(|e| CitraError::PlatformError(e.to_string()))?;
    let mut vec = vec![0; size_of::<T>() * accessor_count];
    file.read_exact(&mut vec)
        .map_err(|e| CitraError::PlatformError(e.to_string()))?;
    for i in 0..accessor_count {
        let data = unsafe {
            let mut uninit: MaybeUninit<T> = MaybeUninit::uninit();
            (uninit.as_mut_ptr() as *mut u8).copy_from_nonoverlapping(
                vec.as_ptr()
                    .offset((size_of::<T>() * i).try_into().unwrap()),
                size_of::<T>(),
            );
            uninit.assume_init()
        };
        func(data, i);
    }

    Ok(())
}

pub fn get_accessor_component_type(
    root: &Root,
    accessor_idx: &Index<Accessor>,
) -> Result<Checked<GenericComponentType>, CitraError> {
    Ok(root
        .accessors
        .get(accessor_idx.value())
        .ok_or(CitraError::PlatformError(
            "Accessor does not exist".to_string(),
        ))?
        .component_type)
}

pub fn verbatim_cast_from_accessor<T>(
    root: &Root,
    bin_chunk_header: &ChunkHeader,
    mut file: &File,
    accessor_idx: &Index<Accessor>,
    accessor_count: usize,
) -> Result<Vec<T, LinearAllocator>, CitraError>
where
    T: Sized,
{
    let accessor = root
        .accessors
        .get(accessor_idx.value())
        .ok_or(CitraError::PlatformError(
            "Accessor does not exist".to_string(),
        ))?;
    let buffer_view_idx = accessor.buffer_view.ok_or(CitraError::PlatformError(
        "Accessor does not contain buffer view idx".to_owned(),
    ))?;
    let buffer_view =
        root.buffer_views
            .get(buffer_view_idx.value())
            .ok_or(CitraError::PlatformError(
                "Buffer View index does not exist".to_owned(),
            ))?;
    let byte_offset: usize = buffer_view
        .byte_offset
        .ok_or(CitraError::PlatformError(
            "Buffer View does not have an offset".to_owned(),
        ))?
        .0
        .try_into()
        .map_err(|_| CitraError::PlatformError("Could not convert u64 to usize".to_owned()))?;

    file.seek(SeekFrom::Start(
        (bin_chunk_header.get_location() + byte_offset) as u64,
    ))
    .map_err(|e| CitraError::PlatformError(e.to_string()))?;
    let mut vec = Vec::with_capacity_in(accessor_count * size_of::<T>(), LinearAllocator);
    vec.resize_with(accessor_count * size_of::<T>(), || 0);
    file.read_exact(&mut vec)
        .map_err(|e| CitraError::PlatformError(e.to_string()))?;
    let mut out: Vec<T, LinearAllocator> =
        unsafe { std::mem::transmute::<Vec<u8, LinearAllocator>, Vec<T, LinearAllocator>>(vec) };
    unsafe {
        out.set_len(accessor_count);
    }

    Ok(out)
}
