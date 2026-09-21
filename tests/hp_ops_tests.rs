use std::path::Path;

use hot_clipboard::hp_ops::{
    copy_file, decode_clipboard_image, default_clip_ts_png, encode_image, ensure_dir,
    file_destination_for_single, same_file,
};

#[test]
fn ensure_dir_creates_missing_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let nested = tmp.path().join("a").join("b");
    assert!(!nested.exists());
    ensure_dir(&nested).expect("ensure_dir should create nested dirs");
    assert!(nested.is_dir());
}

#[test]
fn ensure_dir_is_ok_when_directory_already_exists() {
    let tmp = tempfile::tempdir().unwrap();
    ensure_dir(tmp.path()).expect("ensure_dir should succeed for existing dir");
    assert!(tmp.path().is_dir());
}

#[test]
fn same_file_is_true_for_identical_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("x.bin");
    std::fs::write(&p, b"x").unwrap();
    assert!(same_file(&p, &p));
}

#[test]
fn same_file_is_false_for_different_files() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.bin");
    let b = tmp.path().join("b.bin");
    std::fs::write(&a, b"a").unwrap();
    std::fs::write(&b, b"b").unwrap();
    assert!(!same_file(&a, &b));
}

#[test]
fn copy_file_copies_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let dest = tmp.path().join("dest.txt");
    std::fs::write(&src, b"payload").unwrap();

    copy_file(&src, &dest, false).expect("copy_file should succeed");
    assert_eq!(std::fs::read(&dest).unwrap(), b"payload");
}

#[test]
fn copy_file_refuses_overwrite_without_force() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let dest = tmp.path().join("dest.txt");
    std::fs::write(&src, b"new").unwrap();
    std::fs::write(&dest, b"old").unwrap();

    let err = copy_file(&src, &dest, false).unwrap_err();
    assert!(err.contains("already exists"));
    assert_eq!(std::fs::read(&dest).unwrap(), b"old");
}

#[test]
fn copy_file_overwrites_with_force() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let dest = tmp.path().join("dest.txt");
    std::fs::write(&src, b"new").unwrap();
    std::fs::write(&dest, b"old").unwrap();

    copy_file(&src, &dest, true).expect("force copy should succeed");
    assert_eq!(std::fs::read(&dest).unwrap(), b"new");
}

#[test]
fn copy_file_skips_same_file() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("same.txt");
    std::fs::write(&src, b"keep").unwrap();
    copy_file(&src, &src, true).expect("same-file copy should be a no-op");
    assert_eq!(std::fs::read(&src).unwrap(), b"keep");
}

#[test]
fn copy_file_errors_when_source_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("missing.txt");
    let dest = tmp.path().join("dest.txt");
    let err = copy_file(&src, &dest, false).unwrap_err();
    assert!(err.contains("Source file no longer exists"));
}

#[test]
fn file_destination_for_single_uses_output_filename() {
    let src = Path::new("/tmp/photo.png");
    let dest = file_destination_for_single(src, "renamed.jpg", false).unwrap();
    assert_eq!(dest, Path::new("renamed.jpg"));
}

#[test]
fn file_destination_for_single_inherits_extension_when_missing() {
    let src = Path::new("/tmp/photo.png");
    let dest = file_destination_for_single(src, "renamed", false).unwrap();
    assert_eq!(dest, Path::new("renamed.png"));
}

#[test]
fn file_destination_for_single_joins_existing_dir_unless_rename() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("photo.png");
    std::fs::write(&src, b"x").unwrap();
    let dest_dir = tmp.path().join("out");
    std::fs::create_dir_all(&dest_dir).unwrap();

    let dest = file_destination_for_single(&src, dest_dir.to_str().unwrap(), false).unwrap();
    assert_eq!(dest, dest_dir.join("photo.png"));

    let renamed = file_destination_for_single(&src, dest_dir.to_str().unwrap(), true).unwrap();
    // With force_rename, treat the directory path as a filename (inherit src ext).
    assert_eq!(
        renamed.file_name().and_then(|s| s.to_str()),
        Some("out.png")
    );
}

#[test]
fn default_clip_ts_png_matches_spec_pattern() {
    let name = default_clip_ts_png();
    assert!(name.starts_with("clip_"));
    assert!(name.ends_with(".png"));
    let rest = name
        .strip_prefix("clip_")
        .unwrap()
        .strip_suffix(".png")
        .unwrap();
    let parts: Vec<&str> = rest.split('_').collect();
    assert_eq!(parts.len(), 2, "expected YYYY-MM-DD_HHmmss, got {name}");
    assert_eq!(parts[0].len(), 10);
    assert_eq!(parts[1].len(), 6);
}

#[test]
fn decode_clipboard_image_reads_png_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("img.png");
    let img = image::RgbImage::from_pixel(2, 2, image::Rgb([1u8, 2, 3]));
    img.save_with_format(&path, image::ImageFormat::Png)
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let decoded = decode_clipboard_image(&bytes);
    assert_eq!(decoded.width(), 2);
    assert_eq!(decoded.height(), 2);
}

#[test]
fn encode_image_writes_png_signature() {
    let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        2,
        2,
        image::Rgb([10u8, 20, 30]),
    ));
    let bytes = encode_image(img, "png");
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn encode_image_writes_jpeg_signature() {
    let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        2,
        2,
        image::Rgb([10u8, 20, 30]),
    ));
    let bytes = encode_image(img, "jpg");
    assert_eq!(&bytes[0..2], &[0xFF, 0xD8]);
}
