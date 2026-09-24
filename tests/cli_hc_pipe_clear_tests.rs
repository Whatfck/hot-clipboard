use std::process::{Command, Stdio};

mod common;
use common::lock_clipboard;

use hot_clipboard::get_text;

#[test]
fn hc_cli_stdin_pipe_copies_text() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    // Any path fixture is fine; we just need a stable cwd.

    let hc_exe = env!("CARGO_BIN_EXE_hc");
    let mut child = Command::new(hc_exe)
        .current_dir(&tmp)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"PIPE TEXT")
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success());

    let text = get_text().unwrap();
    assert_eq!(text, Some("PIPE TEXT".to_string()));
}

#[test]
fn hc_cli_clear_clears_text() {
    let _guard = lock_clipboard();

    // First write some text via hc stdin so clipboard is non-empty.
    {
        let hc_exe = env!("CARGO_BIN_EXE_hc");
        let tmp = tempfile::tempdir().unwrap();
        let mut child = Command::new(hc_exe)
            .current_dir(&tmp)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        use std::io::Write;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"TO_CLEAR")
            .unwrap();
        let status = child.wait().unwrap();
        assert!(status.success());
    }

    assert_eq!(get_text().unwrap(), Some("TO_CLEAR".to_string()));

    // Now clear.
    let hc_exe = env!("CARGO_BIN_EXE_hc");
    let tmp = tempfile::tempdir().unwrap();
    let output = Command::new(hc_exe)
        .current_dir(&tmp)
        .arg("-c")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(get_text().unwrap(), None);
}

#[test]
fn hc_cli_invalid_utf8_fails_without_panic() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let hc_exe = env!("CARGO_BIN_EXE_hc");
    let mut child = Command::new(hc_exe)
        .current_dir(&tmp)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"valid\xFF")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("stdin is not valid UTF-8"),
        "stderr: {stderr}"
    );
    assert!(!stderr.contains("panicked"), "stderr: {stderr}");
}
