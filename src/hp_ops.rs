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

    if src.is_dir() {
        let source_root = src
            .canonicalize()
            .map_err(|e| format!("Error: Failed to inspect '{}': {}", src.display(), e))?;
        let destination_root = if dest.exists() {
            dest.canonicalize()
                .map_err(|e| format!("Error: Failed to inspect '{}': {}", dest.display(), e))?
        } else {
            dest.parent()
                .unwrap_or_else(|| Path::new("."))
                .canonicalize()
                .map_err(|e| format!("Error: Failed to inspect '{}': {}", dest.display(), e))?
                .join(dest.file_name().ok_or_else(|| {
                    format!("Error: Destination has no filename: '{}'", dest.display())
                })?)
        };

        if destination_root.starts_with(&source_root) {
            return Err(format!(
                "Error: Cannot copy directory '{}' into itself at '{}'.",
                src.display(),
                dest.display()
            ));
        }

        if dest.exists() && !dest.is_dir() {
            return Err(format!(
                "Error: Destination '{}' is not a directory.",
                dest.display()
            ));
        }

        fs::create_dir_all(dest).map_err(|e| {
            format!(
                "Error: Failed to create directory '{}': {}",
                dest.display(),
                e
            )
        })?;

        for entry in fs::read_dir(src)
            .map_err(|e| format!("Error: Failed to read '{}': {}", src.display(), e))?
        {
            let entry = entry.map_err(|e| {
                format!(
                    "Error: Failed to read directory entry in '{}': {}",
                    src.display(),
                    e
                )
            })?;
            copy_file(&entry.path(), &dest.join(entry.file_name()), force)?;
        }

        println!("✓ Pasted directory {}", dest.display());
        return Ok(());
    }

    if dest.exists() && !force {
        return Err(format!(
            "Error: File '{}' already exists. Use --force to overwrite.",
            dest.display()
        ));
    }

    let source_ext = src
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let dest_ext = dest
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");

    let bytes = if image_ops::looks_like_image_ext(source_ext)
        && image_ops::looks_like_image_ext(dest_ext)
        && !source_ext.eq_ignore_ascii_case(dest_ext)
    {
        let converted = image_ops::convert_or_copy_file(src, dest, dest_ext)?;
        fs::write(dest, &converted)
            .map_err(|e| format!("Error: Failed to copy '{}': {}", src.display(), e))?;
        converted.len() as u64
    } else {
        fs::copy(src, dest)
            .map_err(|e| format!("Error: Failed to copy '{}': {}", src.display(), e))?
    };
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

pub fn batch_extension_from_pattern(pattern: &str) -> Option<&str> {
    let extension = pattern.strip_prefix("*.")?;
    if !extension.is_empty()
        && extension
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        Some(extension)
    } else {
        None
    }
}

pub fn destination_for_multiple_files(
    output: Option<&str>,
    dir: Option<&str>,
    rename: bool,
    count: usize,
) -> Result<PathBuf, String> {
    if let Some(explicit_dir) = dir {
        return Ok(PathBuf::from(explicit_dir));
    }

    if let Some(output) = output {
        let path = Path::new(output);
        if path.is_dir() && !rename {
            return Ok(path.to_path_buf());
        }
    }

    Err(format!(
        "Error: Clipboard contains {} files. Specify a directory: hp -d <dir>",
        count
    ))
}

pub fn destination_for_single_file(
    src: &Path,
    output: Option<&str>,
    dir: Option<&str>,
    rename: bool,
) -> Result<PathBuf, String> {
    if let Some(dir) = dir {
        let destination_dir = Path::new(dir);
        if rename {
            if let Some(output) = output {
                let destination = file_destination_for_single(src, output, true)?;
                let filename = destination
                    .file_name()
                    .ok_or_else(|| "Output destination has no filename".to_string())?;
                return Ok(destination_dir.join(filename));
            }
        }

        let filename = src
            .file_name()
            .ok_or_else(|| "Source has no filename".to_string())?;
        return Ok(destination_dir.join(filename));
    }

    Ok(match output {
        Some(output) => file_destination_for_single(src, output, rename)?,
        None => Path::new(".").join(
            src.file_name()
                .ok_or_else(|| "Source has no filename".to_string())?,
        ),
    })
}

pub fn default_clip_ts_png() -> String {
    use chrono::Local;
    // SPEC: clip_YYYY-MM-DD_HHmmss.png
    Local::now().format("clip_%Y-%m-%d_%H%M%S.png").to_string()
}

pub fn decode_clipboard_image(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    image_ops::decode_image(bytes)
}

pub fn encode_image(img: image::DynamicImage, ext: &str) -> Result<Vec<u8>, String> {
    let ext = ext.to_ascii_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "tiff" | "tif" => image_ops::encode_image(img, &ext),
        other => Err(format!("Unsupported output image extension: {}", other)),
    }
}
