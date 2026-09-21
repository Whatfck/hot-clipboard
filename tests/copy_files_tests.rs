use hot_clipboard::{copy_files, get_file_urls};

mod common;
use common::lock_clipboard;

#[test]
fn test_copy_files_succeeds() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("test.txt");
    std::fs::write(&file, b"test content").unwrap();

    let result = copy_files(&[file.to_string_lossy().to_string()]);
    assert!(result.is_ok(), "copy_files should succeed");

    let got = get_file_urls().expect("get_file_urls should work");
    assert!(got.is_some(), "Should have file URLs in clipboard");
}

#[test]
fn test_copy_files_multiple_files() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let file1 = tmp.path().join("file1.txt");
    let file2 = tmp.path().join("file2.txt");
    std::fs::write(&file1, b"content1").unwrap();
    std::fs::write(&file2, b"content2").unwrap();

    copy_files(&[
        file1.to_string_lossy().to_string(),
        file2.to_string_lossy().to_string(),
    ])
    .expect("copy_files should succeed");

    let got = get_file_urls().expect("get_file_urls should work");
    assert!(got.is_some(), "Should have file URLs in clipboard");
    let urls = got.unwrap();
    assert_eq!(urls.len(), 2, "Should have copied both files");
}

#[test]
fn test_copy_files_rejects_missing_file() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("missing.txt");

    let result = copy_files(&[missing.to_string_lossy().to_string()]);
    assert!(result.is_err(), "Should error on missing file");
    assert!(result.unwrap_err().contains("No such file or directory"));
}

#[test]
fn test_copy_files_returns_canonical_paths() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let real_file = tmp.path().join("real.txt");
    let link_file = tmp.path().join("link.txt");

    std::fs::write(&real_file, b"payload").unwrap();
    std::os::unix::fs::symlink(&real_file, &link_file).unwrap();

    copy_files(&[link_file.to_string_lossy().to_string()]).expect("copy_files should succeed");

    let got = get_file_urls().expect("get_file_urls should work");
    assert!(got.is_some(), "Should have file URLs in clipboard");
    let urls = got.unwrap();
    assert_eq!(urls.len(), 1, "Should have one file URL");

    // The path should be canonicalized (the symlink should resolve)
    assert_eq!(urls[0], real_file.canonicalize().unwrap().to_string_lossy());
}
