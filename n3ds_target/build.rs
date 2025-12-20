use std::{ffi::OsStr, fs, path::Path, process::Command};

fn copy_write_into_overwrite(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    ignore_extensions: &Vec<&OsStr>,
) -> Result<(), std::io::Error> {
    fs::create_dir_all(&dst)?;
    'entry_loop: for entry in fs::read_dir(src)? {
        let entry = entry?;
        for ig_ext in ignore_extensions {
            if let Some(ext) = &entry.path().extension()
                && ext == ig_ext
            {
                continue 'entry_loop;
            }
        }
        let t = entry.file_type()?;
        if t.is_dir() {
            copy_write_into_overwrite(
                entry.path(),
                dst.as_ref().join(entry.file_name()),
                ignore_extensions,
            )?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn compile_shaders(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), std::io::Error> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let t = entry.file_type()?;
        if t.is_dir() {
            compile_shaders(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else if let Some(ext) = entry.path().extension()
            && ext == "pica"
        {
            Command::new("picasso")
                .arg(entry.path())
                .arg("-o")
                .arg(
                    dst.as_ref().join(
                        entry
                            .path()
                            .file_stem()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned()
                            + ".shbin",
                    ),
                )
                .output()?;
        }
    }
    Ok(())
}

fn convert_all_png_to_t3x(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<(), std::io::Error> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let t = entry.file_type()?;
        if t.is_dir() {
            convert_all_png_to_t3x(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else if let Some(ext) = entry.path().extension()
            && ext == "png"
        {
            Command::new("tex3ds")
                .arg("-f")
                .arg("auto-etc1")
                .arg("-z")
                .arg("auto")
                .arg(entry.path())
                .arg("-o")
                .arg(
                    dst.as_ref().join(
                        entry
                            .path()
                            .file_stem()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned()
                            + ".t3x",
                    ),
                )
                .output()?;
        }
    }
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let out_dir = "../target/armv6k-nintendo-3ds";

    println!("cargo:rerun-if-env-changed=PATH");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../assets");
    println!("cargo:rerun-if-changed=../icon.png");

    let full_romfs = Path::new(out_dir).join("full_romfs");
    println!("cargo:rerun-if-changed={}", full_romfs.as_path().display());

    if fs::exists("../icon.png")? {
        fs::copy("../icon.png", "./icon.png")?;
    }

    // grab platfom-agnostic and platform specific data while ignoring special file formats
    copy_write_into_overwrite(
        "../assets",
        full_romfs.clone(),
        &vec![OsStr::new("pica"), OsStr::new("png")],
    )?;

    // process shaders
    compile_shaders("../assets", full_romfs.clone())?;

    // process pngs
    convert_all_png_to_t3x("../assets", full_romfs.clone())?;

    Ok(())
}
