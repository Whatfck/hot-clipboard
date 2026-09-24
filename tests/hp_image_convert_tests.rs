use std::process::Command;

mod common;
use common::{create_test_png, lock_clipboard};

use hot_clipboard::{copy_bitmap_image, copy_files};

#[test]
fn hp_image_rename_infers_png_extension() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let img = create_test_png(&tmp.path().join("img.png"));

    copy_bitmap_image(img.to_string_lossy().as_ref()).expect("copy_bitmap_image should succeed");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let status = Command::new(hp_exe)
        .current_dir(&tmp)
        .args(["renamed"]) // no ext -> should default png
        .status()
        .unwrap();

    assert!(status.success());
    assert!(tmp.path().join("renamed.png").exists());
}

#[test]
fn hp_image_convert_png_to_jpg() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let img = create_test_png(&tmp.path().join("img.png"));

    copy_bitmap_image(img.to_string_lossy().as_ref()).expect("copy_bitmap_image should succeed");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let status = Command::new(hp_exe)
        .current_dir(&tmp)
        .args(["renamed.jpg", "--force"]) // force so reruns won't fail
        .status()
        .unwrap();

    assert!(status.success());

    let out = tmp.path().join("renamed.jpg");
    assert!(out.exists());

    let bytes = std::fs::read(out).unwrap();
    assert!(bytes.len() >= 2);
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xD8);
}

#[test]
fn hp_finder_file_convert_png_to_jpg() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let img = create_test_png(&tmp.path().join("source.png"));
    copy_files(&[img.to_string_lossy().to_string()]).expect("copy_files should succeed");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let status = Command::new(hp_exe)
        .current_dir(&tmp)
        .args(["converted.jpg", "--force"])
        .status()
        .unwrap();

    assert!(status.success());
    let bytes = std::fs::read(tmp.path().join("converted.jpg")).unwrap();
    let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg)
        .expect("Finder image output should be valid JPEG");
    assert_eq!((decoded.width(), decoded.height()), (2, 2));
}

#[test]
fn hp_image_without_args_writes_clip_png() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let img = create_test_png(&tmp.path().join("img.png"));

    copy_bitmap_image(img.to_string_lossy().as_ref()).expect("copy_bitmap_image should succeed");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let status = Command::new(hp_exe).current_dir(&tmp).status().unwrap();

    assert!(status.success());

    let entries: Vec<_> = std::fs::read_dir(&tmp)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    let has_clip_png = entries.iter().any(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with("clip_") && s.ends_with(".png"))
            .unwrap_or(false)
    });

    assert!(has_clip_png);
}
