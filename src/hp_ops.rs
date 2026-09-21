use std::fs;
use std::path::{Path, PathBuf};

use crate::image_ops;

pub fn ensure_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| {
            format!(
                "Error: Failed to create directory '{}': {}",
                path.display(),
                e
            )
        })?;
    }
    Ok(())
}

pub fn same_file(src: &Path, dest: &Path) -> bool {
    src.canonicalize()
        .ok()
        .zip(dest.canonicalize().ok())
        .map(|(a, b)| a == b)
        .unwrap_or(false)
}

pub fn copy_file(src: &Path, dest: &Path, force: bool) -> Result<(), String> {
    if !src.exists() {
        return Err(format!(
            "Error: Source file no longer exists: '{}'",
            src.display()
        ));
    }

    if same_file(src, dest) {
        println!("✓ Already at {}", dest.display());
        return Ok(());
    }

    if dest.exists() && !force {
        return Err(format!(
            "Error: File '{}' already exists. Use --force to overwrite.",
            dest.display()
        ));
    }

    let bytes = fs::copy(src, dest)
        .map_err(|e| format!("Error: Failed to copy '{}': {}", src.display(), e))?;
    println!("✓ Pasted {} ({} bytes)", dest.display(), bytes);
    Ok(())
}

pub fn file_destination_for_single(
    src: &Path,
    output: &str,
    force_rename: bool,
) -> Result<PathBuf, String> {
    let out_path = Path::new(output);

    // If output exists as directory and rename is NOT set -> copy inside that directory.
    if !force_rename && out_path.is_dir() {
        let filename = src
            .file_name()
            .ok_or_else(|| "Source has no filename".to_string())?;
        return Ok(out_path.join(filename));
    }

    // Treat output as filename; if no extension, inherit from src.
    let mut dest = out_path.to_path_buf();
    if dest.extension().is_none() {
        if let Some(src_ext) = src.extension() {
            dest.set_extension(src_ext);
        }
    }
    Ok(dest)
}

pub fn default_clip_ts_png() -> String {
    use chrono::Local;
    // SPEC: clip_YYYY-MM-DD_HHmmss.png
    Local::now().format("clip_%Y-%m-%d_%H%M%S.png").to_string()
}

pub fn decode_clipboard_image(bytes: &[u8]) -> image::DynamicImage {
    image_ops::decode_image(bytes).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    })
}

pub fn encode_image(img: image::DynamicImage, ext: &str) -> Vec<u8> {
    let ext = ext.to_ascii_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" => image_ops::encode_image(img, &ext).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }),
        other => {
            eprintln!("Error: Unsupported output image extension: {}", other);
            std::process::exit(1);
        }
    }
}
