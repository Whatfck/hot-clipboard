mod common;
use common::lock_clipboard;

use hot_clipboard::{copy_text, get_text};

use std::process::Command;

#[test]
fn hp_text_writes_file_when_output_given() {
    let _guard = lock_clipboard();

    copy_text("hello text").expect("copy_text should work");

    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("out.txt");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let output = Command::new(hp_exe)
        .current_dir(&tmp)
        .arg(out_path.as_os_str())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(out_path.exists());
    assert_eq!(std::fs::read_to_string(&out_path).unwrap(), "hello text");
}

#[test]
fn hp_text_refuses_overwrite_without_force() {
    let _guard = lock_clipboard();

    copy_text("new text").expect("copy_text should work");

    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("out.txt");
    std::fs::write(&out_path, "old").unwrap();

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let output = Command::new(hp_exe)
        .current_dir(&tmp)
        .arg(out_path.as_os_str())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"), "stderr was: {}", stderr);

    // Should remain unchanged.
    assert_eq!(std::fs::read_to_string(&out_path).unwrap(), "old");

    // Now allow with --force.
    let output2 = Command::new(hp_exe)
        .current_dir(&tmp)
        .arg("--force")
        .arg(out_path.as_os_str())
        .output()
        .unwrap();

    assert!(output2.status.success());
    assert_eq!(std::fs::read_to_string(&out_path).unwrap(), "new text");

    // Clipboard should still contain text.
    assert_eq!(get_text().unwrap(), Some("new text".to_string()));
}
