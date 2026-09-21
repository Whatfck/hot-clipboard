use std::path::Path;
use std::process::Command;

mod common;
use common::lock_clipboard;

use hot_clipboard::copy_files;

fn abs_path(p: &Path) -> String {
    p.canonicalize().unwrap().to_string_lossy().to_string()
}

#[test]
fn hp_uses_positional_existing_dir_for_multiple_files() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();

    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    let a = src_dir.join("a.bin");
    std::fs::write(&a, b"aaa").unwrap();

    let b = src_dir.join("b.txt");
    std::fs::write(&b, b"bbb").unwrap();

    copy_files(&[abs_path(&a), abs_path(&b)]).expect("copy_files should work");

    let dest_dir = tmp.path().join("dest");
    std::fs::create_dir_all(&dest_dir).unwrap();

    let hp_exe = env!("CARGO_BIN_EXE_hp");

    // SPEC A.3: if positional arg exists as directory, paste inside it.
    let status = Command::new(hp_exe)
        .current_dir(&tmp)
        .arg(dest_dir.to_string_lossy().as_ref())
        .status()
        .unwrap();

    assert!(status.success(), "hp should succeed");

    assert!(dest_dir.join("a.bin").exists());
    assert!(dest_dir.join("b.txt").exists());
    assert_eq!(std::fs::read(dest_dir.join("a.bin")).unwrap(), b"aaa");
    assert_eq!(std::fs::read(dest_dir.join("b.txt")).unwrap(), b"bbb");
}
