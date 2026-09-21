use std::path::Path;

mod common;
use common::lock_clipboard;

use hot_clipboard::{copy_files, get_file_urls};

fn abs_path(p: &Path) -> String {
    p.canonicalize().unwrap().to_string_lossy().to_string()
}

#[test]
fn copy_files_canonicalizes_symlinks() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();

    let real_dir = tmp.path().join("real");
    std::fs::create_dir_all(&real_dir).unwrap();

    let real_file = real_dir.join("file.bin");
    std::fs::write(&real_file, b"payload").unwrap();

    let link_file = tmp.path().join("link.bin");

    // Create symlink. On some systems symlink might fail depending on permissions;
    // if it fails, we still want a deterministic test failure rather than panic.
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&real_file, &link_file).unwrap();
    }

    let link_str = link_file.to_string_lossy().to_string();
    copy_files(&[link_str]).expect("copy_files should succeed");

    let got = get_file_urls()
        .unwrap()
        .expect("clipboard should contain file urls");
    assert_eq!(got.len(), 1);

    let got0 = &got[0];
    let expected = abs_path(&real_file);

    // SPEC: should store canonical path, not the symlink path.
    assert_eq!(got0, &expected);
}
