use std::path::Path;
use std::process::Command;

mod common;
use common::lock_clipboard;

use hot_clipboard::{get_file_urls, get_text};

fn abs_path(p: &Path) -> String {
    p.canonicalize().unwrap().to_string_lossy().to_string()
}

fn base_names(paths: &[String]) -> Vec<String> {
    let mut v: Vec<String> = paths
        .iter()
        .filter_map(|p| {
            Path::new(p)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
        })
        .collect();
    v.sort();
    v
}

#[test]
fn hc_cli_copies_multiple_files_into_clipboard() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let f1 = tmp.path().join("file1.bin");
    std::fs::write(&f1, b"abc").unwrap();

    let f2 = tmp.path().join("another.txt");
    std::fs::write(&f2, b"def").unwrap();

    let hc_exe = env!("CARGO_BIN_EXE_hc");
    let status = Command::new(hc_exe)
        .current_dir(&tmp)
        .args([abs_path(&f1).as_str(), abs_path(&f2).as_str()])
        .status()
        .unwrap();

    assert!(status.success());

    let got = get_file_urls()
        .unwrap()
        .expect("clipboard should contain file urls");
    assert_eq!(
        base_names(&got),
        base_names(&[abs_path(&f1), abs_path(&f2)])
    );
}

#[test]
fn hc_cli_file_args_take_priority_over_stdin_pipe() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let f1 = tmp.path().join("file1.bin");
    std::fs::write(&f1, b"abc").unwrap();

    // stdin pipe contains text, but hc should treat stdin as unrelated when file args exist.
    let mut child = Command::new(env!("CARGO_BIN_EXE_hc"))
        .current_dir(&tmp)
        .arg(abs_path(&f1))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();

    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"SHOULD_NOT_BE_COPIED")
        .unwrap();
    let status = child.wait().unwrap();
    assert!(status.success());

    let file_urls = get_file_urls()
        .unwrap()
        .expect("clipboard should contain file urls");
    assert_eq!(file_urls.len(), 1);

    let text = get_text().unwrap();
    assert!(
        text.is_none(),
        "text should not be copied when file args are present"
    );
}
