use std::process::Command;

mod common;
use common::lock_clipboard;

#[test]
fn hc_cli_nonexistent_file_fails_with_expected_message() {
    let _guard = lock_clipboard();

    let hc_exe = env!("CARGO_BIN_EXE_hc");
    let tmp = tempfile::tempdir().unwrap();

    let bogus = tmp.path().join("does_not_exist.txt");
    let output = Command::new(hc_exe)
        .current_dir(&tmp)
        .arg(bogus.to_string_lossy().as_ref())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    // SPEC: Error: No such file or directory: 'X'
    assert!(
        stderr.contains("No such file or directory"),
        "stderr was: {}",
        stderr
    );
}
