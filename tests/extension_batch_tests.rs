use std::path::Path;
use std::process::Command;

mod common;
use common::{create_test_png, lock_clipboard};

use hot_clipboard::copy_files;

fn abs_path(p: &Path) -> String {
    p.canonicalize().unwrap().to_string_lossy().to_string()
}

#[test]
fn hp_batch_changes_text_extension_and_preserves_content() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    let a = src_dir.join("a.md");
    std::fs::write(&a, b"aaa").unwrap();

    let b = src_dir.join("b.md");
    std::fs::write(&b, b"bbb").unwrap();

    copy_files(&[abs_path(&a), abs_path(&b)]).expect("copy_files should succeed");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let status = Command::new(hp_exe)
        .current_dir(&tmp)
        .args(["*.txt"])
        .arg("--force")
        .status()
        .unwrap();

    assert!(status.success());
    assert_eq!(std::fs::read(tmp.path().join("a.txt")).unwrap(), b"aaa");
    assert_eq!(std::fs::read(tmp.path().join("b.txt")).unwrap(), b"bbb");
}

#[test]
fn hp_batch_converts_real_png_to_jpeg() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let png = create_test_png(&tmp.path().join("photo.png"));

    copy_files(&[abs_path(&png)]).expect("copy_files should succeed");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let status = Command::new(hp_exe)
        .current_dir(&tmp)
        .args(["*.jpg", "--force"])
        .status()
        .unwrap();

    assert!(status.success());
    let out = tmp.path().join("photo.jpg");
    assert!(out.exists());
    let bytes = std::fs::read(&out).unwrap();
    let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg)
        .expect("batch output should be valid JPEG");
    assert_eq!((decoded.width(), decoded.height()), (2, 2));
}
