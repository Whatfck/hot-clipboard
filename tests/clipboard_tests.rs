use hot_clipboard::{copy_files, copy_text, get_file_urls, get_text};

mod common;
use common::lock_clipboard;

#[test]
fn copy_text_and_get_text_roundtrip() {
    let _guard = lock_clipboard();

    copy_text("hello roundtrip").expect("copy_text should work");

    let got = get_text().expect("get_text should work");
    assert_eq!(got, Some("hello roundtrip".to_string()));
}

#[test]
fn copy_files_and_get_file_urls_roundtrip() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.bin");
    let b = tmp.path().join("b.txt");
    std::fs::write(&a, b"aaa").unwrap();
    std::fs::write(&b, b"bbb").unwrap();

    copy_files(&[
        a.to_string_lossy().to_string(),
        b.to_string_lossy().to_string(),
    ])
    .expect("copy_files should work");

    let got = get_file_urls().expect("get_file_urls should work");
    assert!(got.is_some());
    let urls = got.unwrap();
    assert_eq!(urls.len(), 2);
}

#[test]
fn get_file_urls_returns_none_when_empty() {
    let _guard = lock_clipboard();

    hot_clipboard::clear_clipboard().expect("clear_clipboard should work");

    let got = get_file_urls().expect("get_file_urls should work");
    assert_eq!(got, None);
}

#[test]
fn get_text_returns_none_when_empty() {
    let _guard = lock_clipboard();

    hot_clipboard::clear_clipboard().expect("clear_clipboard should work");

    let got = get_text().expect("get_text should work");
    assert_eq!(got, None);
}

#[test]
fn copy_text_overwrites_previous_clipboard() {
    let _guard = lock_clipboard();

    copy_text("first text").expect("copy_text should work");
    assert_eq!(
        get_text().expect("get_text should work"),
        Some("first text".to_string())
    );

    copy_text("second text").expect("copy_text should work");
    assert_eq!(
        get_text().expect("get_text should work"),
        Some("second text".to_string())
    );
}

#[test]
fn copy_files_preserves_multiple_paths() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let paths: Vec<String> = (0..3)
        .map(|i| {
            let p = tmp.path().join(format!("file_{}.txt", i));
            std::fs::write(&p, format!("content-{}", i)).unwrap();
            p.to_string_lossy().to_string()
        })
        .collect();

    copy_files(&paths).expect("copy_files should work");

    let got = get_file_urls().expect("get_file_urls should work");
    assert!(got.is_some());
    let urls = got.unwrap();
    assert_eq!(urls.len(), 3);
}

#[test]
fn copy_files_rejects_missing_file() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("does_not_exist.txt");

    let result = copy_files(&[missing.to_string_lossy().to_string()]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No such file or directory"));
}

#[test]
fn copy_text_handles_empty_string() {
    let _guard = lock_clipboard();

    copy_text("").expect("copy_text should work with empty string");

    // Empty string is valid content
    let got = get_text().expect("get_text should work");
    assert_eq!(got, Some("".to_string()));
}

#[test]
fn get_file_urls_returns_canonical_paths() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let real = tmp.path().join("real.txt");
    let link = tmp.path().join("link.txt");
    std::fs::write(&real, b"payload").unwrap();

    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &link).unwrap();

    copy_files(&[link.to_string_lossy().to_string()]).expect("copy_files should work");

    let got = get_file_urls().expect("get_file_urls should work");
    assert!(got.is_some());
    let urls = got.unwrap();
    assert_eq!(urls.len(), 1);
    // Should be canonicalized
    assert_eq!(urls[0], real.canonicalize().unwrap().to_string_lossy());
}

#[test]
fn copy_files_multiple_mixed_absolute_relative() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let abs = tmp.path().join("abs.txt");
    std::fs::write(&abs, b"abs").unwrap();

    std::env::set_current_dir(&tmp).unwrap();

    let rel = "abs.txt".to_string();
    let abs_str = abs.to_string_lossy().to_string();

    copy_files(&[abs_str.clone(), rel]).expect("copy_files should work");

    let got = get_file_urls().expect("get_file_urls should work");
    assert!(got.is_some());
    let urls = got.unwrap();
    assert_eq!(urls.len(), 2);
}
