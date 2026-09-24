use std::path::Path;
use std::process::Command;

mod common;
use common::{create_test_png, lock_clipboard};

use hot_clipboard::{copy_bitmap_image, copy_files};

fn abs_path(path: &Path) -> String {
    path.canonicalize().unwrap().to_string_lossy().to_string()
}

#[test]
fn hp_dir_copies_single_file_and_creates_destination() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.txt");
    std::fs::write(&source, b"payload").unwrap();
    copy_files(&[abs_path(&source)]).expect("copy_files should work");

    let destination = tmp.path().join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .args(["--dir", destination.to_string_lossy().as_ref()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        std::fs::read(destination.join("source.txt")).unwrap(),
        b"payload"
    );
}

#[test]
fn hp_dir_copies_multiple_files_and_creates_destination() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let first = tmp.path().join("first.txt");
    let second = tmp.path().join("second.txt");
    std::fs::write(&first, b"first").unwrap();
    std::fs::write(&second, b"second").unwrap();
    copy_files(&[abs_path(&first), abs_path(&second)]).expect("copy_files should work");

    let destination = tmp.path().join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .args(["--dir", destination.to_string_lossy().as_ref()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        std::fs::read(destination.join("first.txt")).unwrap(),
        b"first"
    );
    assert_eq!(
        std::fs::read(destination.join("second.txt")).unwrap(),
        b"second"
    );
}

#[test]
fn hp_multiple_files_without_destination_fails_with_guidance() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let first = tmp.path().join("first.txt");
    let second = tmp.path().join("second.txt");
    std::fs::write(&first, b"first").unwrap();
    std::fs::write(&second, b"second").unwrap();
    copy_files(&[abs_path(&first), abs_path(&second)]).expect("copy_files should work");

    let output = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Clipboard contains 2 files. Specify a directory: hp -d <dir>"),
        "stderr: {stderr}"
    );
}

#[test]
fn hp_rename_treats_existing_directory_as_filename() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.txt");
    std::fs::write(&source, b"payload").unwrap();
    copy_files(&[abs_path(&source)]).expect("copy_files should work");
    std::fs::create_dir(tmp.path().join("renamed")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .args(["--rename", "renamed"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        std::fs::read(tmp.path().join("renamed.txt")).unwrap(),
        b"payload"
    );
    assert!(!tmp.path().join("renamed/source.txt").exists());
}

#[test]
fn hp_reports_deleted_source_file() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.txt");
    std::fs::write(&source, b"payload").unwrap();
    copy_files(&[abs_path(&source)]).expect("copy_files should work");
    std::fs::remove_file(&source).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .args(["--dir", "out"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Source file no longer exists"),
        "stderr: {stderr}"
    );
}

#[test]
fn hp_force_overwrites_file_destination() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.txt");
    let destination = tmp.path().join("destination.txt");
    std::fs::write(&source, b"new").unwrap();
    std::fs::write(&destination, b"old").unwrap();
    copy_files(&[abs_path(&source)]).unwrap();

    let without_force = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .arg(&destination)
        .output()
        .unwrap();
    assert_eq!(without_force.status.code(), Some(1));
    assert_eq!(std::fs::read(&destination).unwrap(), b"old");

    let with_force = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .args(["--force", destination.to_string_lossy().as_ref()])
        .output()
        .unwrap();
    assert!(with_force.status.success());
    assert_eq!(std::fs::read(&destination).unwrap(), b"new");
}

#[test]
fn hp_force_overwrites_batch_destination() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.txt");
    std::fs::write(&source, b"new").unwrap();
    copy_files(&[abs_path(&source)]).unwrap();
    std::fs::write(tmp.path().join("source.out"), b"old").unwrap();

    let without_force = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .arg("*.out")
        .output()
        .unwrap();
    assert_eq!(without_force.status.code(), Some(1));
    assert_eq!(
        std::fs::read(tmp.path().join("source.out")).unwrap(),
        b"old"
    );

    let with_force = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .args(["--force", "*.out"])
        .output()
        .unwrap();
    assert!(with_force.status.success());
    assert_eq!(
        std::fs::read(tmp.path().join("source.out")).unwrap(),
        b"new"
    );
}

#[test]
fn hp_handles_unicode_and_spaces_in_file_paths() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let source_dir = tmp.path().join("origen con espacio");
    std::fs::create_dir(&source_dir).unwrap();
    let source = source_dir.join("café.txt");
    std::fs::write(&source, "contenido").unwrap();
    copy_files(&[abs_path(&source)]).unwrap();

    let destination = tmp.path().join("destino con espacio");
    let output = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .args(["--dir", destination.to_string_lossy().as_ref()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        std::fs::read(destination.join("café.txt")).unwrap(),
        b"contenido"
    );
}

#[test]
fn hp_force_overwrites_raw_image_destination() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let image = create_test_png(&tmp.path().join("image.png"));
    copy_bitmap_image(image.to_string_lossy().as_ref()).unwrap();

    let destination = tmp.path().join("output.png");
    std::fs::write(&destination, b"old").unwrap();
    let without_force = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .arg(&destination)
        .output()
        .unwrap();
    assert_eq!(without_force.status.code(), Some(1));
    assert_eq!(std::fs::read(&destination).unwrap(), b"old");

    let with_force = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .args(["--force", destination.to_string_lossy().as_ref()])
        .output()
        .unwrap();
    assert!(with_force.status.success());
    let decoded = image::open(&destination).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (2, 2));
}
