use std::path::Path;
use std::process::Command;

mod common;
use common::{create_test_png, lock_clipboard};

use hot_clipboard::copy_files;

fn abs_path(p: &Path) -> String {
    p.canonicalize().unwrap().to_string_lossy().to_string()
}

#[test]
fn hp_batch_changes_extension() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    let a = src_dir.join("a.png");
    std::fs::write(&a, b"aaa").unwrap();

    let b = src_dir.join("b.txt");
    std::fs::write(&b, b"bbb").unwrap();

    copy_files(&[abs_path(&a), abs_path(&b)]).expect("copy_files should succeed");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let status = Command::new(hp_exe)
        .current_dir(&tmp)
        .args(["*.jpg"])
        .arg("--force")
        .status()
        .unwrap();

    assert!(status.success());
    assert!(tmp.path().join("a.jpg").exists());
    assert!(tmp.path().join("b.jpg").exists());
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
    assert!(bytes.len() >= 2);
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xD8);
}
