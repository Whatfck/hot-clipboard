use std::process::Command;

use objc2_app_kit::NSPasteboard;
use objc2_foundation::{NSData, NSString};

mod common;
use common::{create_test_png, lock_clipboard};

use hot_clipboard::{copy_bitmap_image, copy_files, copy_raw_bytes};

#[test]
fn hp_prioritizes_files_over_image_and_text() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let source = tmp.path().join("source.txt");
    std::fs::write(&source, b"file payload").unwrap();
    copy_files(&[source.to_string_lossy().to_string()]).unwrap();

    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        let text = NSString::from_str("text payload");
        pasteboard.setString_forType(&text, &NSString::from_str("public.utf8-plain-text"));
        let image = NSData::with_bytes(&[0x89, 0x50, 0x4e, 0x47]);
        pasteboard.setData_forType(Some(&image), objc2_app_kit::NSPasteboardTypePNG);
    }

    let destination = tmp.path().join("out");
    let output = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .args(["--dir", destination.to_string_lossy().as_ref()])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        std::fs::read(destination.join("source.txt")).unwrap(),
        b"file payload"
    );
}

#[test]
fn hp_prioritizes_image_over_text() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    let image_path = create_test_png(&tmp.path().join("image.png"));
    copy_bitmap_image(image_path.to_string_lossy().as_ref()).unwrap();

    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        let text = NSString::from_str("text payload");
        pasteboard.setString_forType(&text, &NSString::from_str("public.utf8-plain-text"));
    }

    let destination = tmp.path().join("out.png");
    let output = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .arg(&destination)
        .output()
        .unwrap();

    assert!(output.status.success());
    let decoded = image::open(&destination).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (2, 2));
}

#[test]
fn hp_prioritizes_text_over_unknown_binary() {
    let _guard = lock_clipboard();
    let tmp = tempfile::tempdir().unwrap();
    copy_raw_bytes(b"binary", "public.data").unwrap();

    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        let text = NSString::from_str("text payload");
        pasteboard.setString_forType(&text, &NSString::from_str("public.utf8-plain-text"));
    }

    let output = Command::new(env!("CARGO_BIN_EXE_hp"))
        .current_dir(&tmp)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"text payload");
}
