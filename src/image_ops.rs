use image::ImageFormat;
use std::io::Cursor;
use std::path::Path;

pub fn encode_image(img: image::DynamicImage, ext: &str) -> Result<Vec<u8>, String> {
    let ext = ext.to_ascii_lowercase();
    let mut buf = Vec::new();
    match ext.as_str() {
        "png" => {
            img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
        }
        "jpg" | "jpeg" => {
            // JPEG encoder does not support alpha.
            let rgb = img.to_rgb8();
            image::DynamicImage::ImageRgb8(rgb)
                .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
        }
        "tiff" | "tif" => {
            img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Tiff)
                .map_err(|e| format!("Failed to encode TIFF: {}", e))?;
        }
        other => return Err(format!("Unsupported output image extension: {}", other)),
    }
    Ok(buf)
}

pub fn decode_image(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    image::load_from_memory(bytes).map_err(|e| format!("Failed to decode clipboard image: {}", e))
}

pub fn looks_like_image_ext(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "tiff" | "tif" | "webp" | "gif" | "bmp"
    )
}

/// Convert a source file to a destination with a new extension.
/// Images are re-encoded when both sides look like image formats; otherwise bytes are copied.
pub fn convert_or_copy_file(src: &Path, _dest: &Path, dest_ext: &str) -> Result<Vec<u8>, String> {
    let src_ext = src
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let dest_ext_l = dest_ext.to_ascii_lowercase();

    if looks_like_image_ext(&src_ext) && looks_like_image_ext(&dest_ext_l) && src_ext != dest_ext_l
    {
        let bytes =
            std::fs::read(src).map_err(|e| format!("Failed to read '{}': {}", src.display(), e))?;
        match decode_image(&bytes) {
            Ok(img) => encode_image(img, &dest_ext_l),
            Err(_) => {
                // Not a real image payload: fall back to a byte copy (rename only).
                Ok(bytes)
            }
        }
    } else {
        std::fs::read(src).map_err(|e| format!("Failed to read '{}': {}", src.display(), e))
    }
}
