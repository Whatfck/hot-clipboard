use hot_clipboard::image_ops;

#[test]
fn looks_like_image_ext_returns_true_for_png() {
    assert!(image_ops::looks_like_image_ext("png"));
}

#[test]
fn looks_like_image_ext_returns_true_for_jpg() {
    assert!(image_ops::looks_like_image_ext("jpg"));
    assert!(image_ops::looks_like_image_ext("jpeg"));
}

#[test]
fn looks_like_image_ext_returns_true_for_tiff() {
    assert!(image_ops::looks_like_image_ext("tiff"));
    assert!(image_ops::looks_like_image_ext("tif"));
}

#[test]
fn looks_like_image_ext_returns_true_for_webp() {
    assert!(image_ops::looks_like_image_ext("webp"));
}

#[test]
fn looks_like_image_ext_returns_true_for_gif() {
    assert!(image_ops::looks_like_image_ext("gif"));
}

#[test]
fn looks_like_image_ext_returns_true_for_bmp() {
    assert!(image_ops::looks_like_image_ext("bmp"));
}

#[test]
fn looks_like_image_ext_returns_false_for_text() {
    assert!(!image_ops::looks_like_image_ext("txt"));
}

#[test]
fn looks_like_image_ext_returns_false_for_rust() {
    assert!(!image_ops::looks_like_image_ext("rs"));
}

#[test]
fn looks_like_image_ext_returns_false_for_unknown() {
    assert!(!image_ops::looks_like_image_ext("xyz"));
}

#[test]
fn looks_like_image_ext_is_case_insensitive() {
    assert!(image_ops::looks_like_image_ext("PNG"));
    assert!(image_ops::looks_like_image_ext("Png"));
    assert!(image_ops::looks_like_image_ext("jpG"));
    assert!(image_ops::looks_like_image_ext("JPEG"));
}

#[test]
fn decode_image_fails_on_invalid_bytes() {
    let result = image_ops::decode_image(b"not an image");
    assert!(result.is_err());
}

#[test]
fn decode_image_succeeds_on_valid_png() {
    let tmp = tempfile::tempdir().unwrap();
    let img = image::RgbImage::from_pixel(2, 2, image::Rgb([255u8, 0, 0]));
    let path = tmp.path().join("test.png");
    img.save_with_format(&path, image::ImageFormat::Png)
        .unwrap();

    let bytes = std::fs::read(&path).unwrap();
    let result = image_ops::decode_image(&bytes);
    assert!(result.is_ok());

    let decoded = result.unwrap();
    assert_eq!(decoded.width(), 2);
    assert_eq!(decoded.height(), 2);
}

#[test]
fn decode_image_succeeds_on_valid_tiff() {
    let tmp = tempfile::tempdir().unwrap();
    let img = image::RgbImage::from_pixel(2, 2, image::Rgb([0u8, 255, 0]));
    let path = tmp.path().join("test.tiff");
    img.save_with_format(&path, image::ImageFormat::Tiff)
        .unwrap();

    let bytes = std::fs::read(&path).unwrap();
    let result = image_ops::decode_image(&bytes);
    assert!(result.is_ok());

    let decoded = result.unwrap();
    assert_eq!(decoded.width(), 2);
    assert_eq!(decoded.height(), 2);
}

#[test]
fn encode_image_to_png() {
    let img = image::RgbImage::from_pixel(2, 2, image::Rgb([100u8, 200, 50]));
    let dyn_img = image::DynamicImage::ImageRgb8(img);

    let result = image_ops::encode_image(dyn_img, "png");
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
    // PNG signature: 89 50 4E 47
    assert_eq!(bytes[0], 0x89);
    assert_eq!(bytes[1], 0x50);
    assert_eq!(bytes[2], 0x4E);
    assert_eq!(bytes[3], 0x47);
}

#[test]
fn encode_image_to_jpeg() {
    let img = image::RgbImage::from_pixel(2, 2, image::Rgb([100u8, 200, 50]));
    let dyn_img = image::DynamicImage::ImageRgb8(img);

    let result = image_ops::encode_image(dyn_img, "jpg");
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
    // JPEG signature: FF D8
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xD8);
}

#[test]
fn encode_image_to_jpeg_uses_rgb() {
    let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([100u8, 200, 50, 255]));
    let dyn_img = image::DynamicImage::ImageRgba8(img);

    // JPEG doesn't support alpha, should convert to RGB automatically
    let result = image_ops::encode_image(dyn_img, "jpeg");
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xD8);
}

#[test]
fn encode_image_to_tiff() {
    let img = image::RgbImage::from_pixel(2, 2, image::Rgb([100u8, 200, 50]));
    let dyn_img = image::DynamicImage::ImageRgb8(img);

    let result = image_ops::encode_image(dyn_img, "tiff");
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn encode_image_returns_error_for_unsupported_ext() {
    let img = image::RgbImage::from_pixel(2, 2, image::Rgb([100u8, 200, 50]));
    let dyn_img = image::DynamicImage::ImageRgb8(img);

    let result = image_ops::encode_image(dyn_img, "bmp");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Unsupported output image extension"));
}

#[test]
fn convert_or_copy_file_converts_png_to_jpg() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("input.png");
    let dest = tmp.path().join("output.jpg");

    let img = image::RgbImage::from_pixel(2, 2, image::Rgb([255u8, 0, 0]));
    img.save_with_format(&src, image::ImageFormat::Png).unwrap();

    let result = image_ops::convert_or_copy_file(&src, &dest, "jpg");
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
    // Verify it's actually JPEG
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xD8);
}

#[test]
fn convert_or_copy_file_converts_png_to_tiff() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("input.png");
    let dest = tmp.path().join("output.tiff");

    let img = image::RgbImage::from_pixel(2, 2, image::Rgb([255u8, 0, 0]));
    img.save_with_format(&src, image::ImageFormat::Png).unwrap();

    let result = image_ops::convert_or_copy_file(&src, &dest, "tiff");
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn convert_or_copy_file_copies_when_both_are_image_but_same_ext() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("input.png");
    let dest = tmp.path().join("output.png");

    let original_content = b"raw image data";
    std::fs::write(&src, original_content).unwrap();

    let result = image_ops::convert_or_copy_file(&src, &dest, "png");
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert_eq!(bytes, original_content);
}

#[test]
fn convert_or_copy_file_copies_when_dest_is_not_image() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("input.png");
    let dest = tmp.path().join("output.txt");

    let original_content = b"raw image data";
    std::fs::write(&src, original_content).unwrap();

    let result = image_ops::convert_or_copy_file(&src, &dest, "txt");
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert_eq!(bytes, original_content);
}

#[test]
fn convert_or_copy_file_copies_when_src_is_not_image() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("input.txt");
    let dest = tmp.path().join("output.png");

    let original_content = b"plain text content";
    std::fs::write(&src, original_content).unwrap();

    let result = image_ops::convert_or_copy_file(&src, &dest, "png");
    assert!(result.is_ok());

    let bytes = result.unwrap();
    assert_eq!(bytes, original_content);
}

#[test]
fn convert_or_copy_file_returns_error_when_src_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("missing.png");
    let dest = tmp.path().join("output.jpg");

    let result = image_ops::convert_or_copy_file(&src, &dest, "jpg");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read"));
}
