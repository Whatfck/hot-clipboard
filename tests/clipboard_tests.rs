use hot_clipboard::{
    clear_clipboard, copy_files, copy_raw_bytes, copy_text, get_clipboard_size, get_file_urls,
    get_text, inspect_clipboard, ClipboardKind,
};
use objc2::runtime::ProtocolObject;
use objc2_app_kit::NSPasteboard;
use objc2_foundation::{NSArray, NSString, NSURL};

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
fn get_text_reads_utf8_type_without_legacy_type() {
    let _guard = lock_clipboard();

    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        let value = NSString::from_str("utf8 only");
        let utf8_type = NSString::from_str("public.utf8-plain-text");
        assert!(pasteboard.setString_forType(&value, &utf8_type));
    }

    assert_eq!(get_text().unwrap(), Some("utf8 only".to_string()));
}

#[test]
fn get_file_urls_decodes_public_file_url_values() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("file with spaces.txt");
    std::fs::write(&file, b"payload").unwrap();
    let file_url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(&file.to_string_lossy())) };

    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        let values = NSArray::from_vec(vec![ProtocolObject::from_retained(file_url)]);
        assert!(pasteboard.writeObjects(&values));
    }

    assert_eq!(
        get_file_urls().unwrap(),
        Some(vec![file.to_string_lossy().to_string()])
    );
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
fn copy_files_keeps_previous_clipboard_when_validation_fails() {
    let _guard = lock_clipboard();

    copy_text("keep this").expect("copy_text should work");
    let tmp = tempfile::tempdir().unwrap();
    let valid = tmp.path().join("valid.txt");
    std::fs::write(&valid, b"valid").unwrap();
    let missing = tmp.path().join("missing.txt");

    let result = copy_files(&[
        valid.to_string_lossy().to_string(),
        missing.to_string_lossy().to_string(),
    ]);

    assert!(result.is_err());
    assert_eq!(get_text().unwrap(), Some("keep this".to_string()));
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
fn clipboard_size_and_kind_cover_text_files_binary_and_empty() {
    let _guard = lock_clipboard();

    copy_text("hello").unwrap();
    assert_eq!(get_clipboard_size().unwrap(), Some(5));
    assert_eq!(inspect_clipboard().unwrap(), ClipboardKind::Text);

    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("size.txt");
    std::fs::write(&file, b"1234").unwrap();
    copy_files(&[file.to_string_lossy().to_string()]).unwrap();
    assert_eq!(get_clipboard_size().unwrap(), Some(4));
    assert_eq!(inspect_clipboard().unwrap(), ClipboardKind::Files);

    copy_raw_bytes(b"binary", "public.data").unwrap();
    assert_eq!(get_clipboard_size().unwrap(), None);
    assert_eq!(inspect_clipboard().unwrap(), ClipboardKind::Binary);

    clear_clipboard().unwrap();
    assert_eq!(get_clipboard_size().unwrap(), None);
    assert_eq!(inspect_clipboard().unwrap(), ClipboardKind::Empty);
}

#[test]
fn copy_files_multiple_mixed_absolute_relative() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let abs = tmp.path().join("abs.txt");
    std::fs::write(&abs, b"abs").unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&tmp).unwrap();

    let rel = "abs.txt".to_string();
    let abs_str = abs.to_string_lossy().to_string();

    copy_files(&[abs_str.clone(), rel]).expect("copy_files should work");

    let got = get_file_urls().expect("get_file_urls should work");
    assert!(got.is_some());
    let urls = got.unwrap();
    assert_eq!(urls.len(), 2);

    std::env::set_current_dir(original_dir).unwrap();
}
