use std::process::Command;

mod common;
use common::lock_clipboard;

use hot_clipboard::{clear_clipboard, copy_raw_bytes};

#[test]
fn hp_unknown_binary_without_filename_errors() {
    let _guard = lock_clipboard();

    copy_raw_bytes(b"\x00\x01\x02\x03not-an-image", "public.data").expect("copy_raw_bytes");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(hp_exe).current_dir(&tmp).output().unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Clipboard contains binary data. Specify a filename: hp <name>"),
        "stderr was: {stderr}"
    );
}

#[test]
fn hp_empty_clipboard_errors() {
    let _guard = lock_clipboard();
    clear_clipboard().expect("clear");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let tmp = tempfile::tempdir().unwrap();
    let out = Command::new(hp_exe).current_dir(&tmp).output().unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Clipboard is empty."),
        "stderr was: {stderr}"
    );
}
