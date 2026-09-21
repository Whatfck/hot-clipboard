use std::path::Path;
use std::process::Command;

mod common;
use common::lock_clipboard;

use hot_clipboard::{copy_files, copy_text};

fn abs_path(p: &Path) -> String {
    p.canonicalize().unwrap().to_string_lossy().to_string()
}

#[test]
fn hp_info_reports_text_as_elements_1() {
    let _guard = lock_clipboard();

    copy_text("hello info").expect("copy_text should work");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let out = Command::new(hp_exe).arg("--info").output().unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        stdout.contains("Clipboard kind: Text"),
        "stdout was: {stdout}"
    );
    assert!(stdout.contains("Clipboard types:"), "stdout was: {stdout}");
    assert!(stdout.contains("Elements: 1"), "stdout was: {stdout}");
    assert!(
        stdout.contains("Approximate bytes:"),
        "stdout was: {stdout}"
    );
}

#[test]
fn hp_info_reports_file_paths_when_clipboard_has_files() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("a.bin");
    std::fs::write(&src, b"abc").unwrap();

    copy_files(&[abs_path(&src)]).expect("copy_files should work");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let out = Command::new(hp_exe)
        .arg("--info")
        .current_dir(&tmp)
        .output()
        .unwrap();
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("File paths:"), "stdout was: {stdout}");
    assert!(
        stdout.contains("Approximate bytes:"),
        "stdout was: {stdout}"
    );
    assert!(stdout.contains("a.bin"), "stdout was: {stdout}");
}
