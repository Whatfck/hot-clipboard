use hot_clipboard::{clear_clipboard, copy_bitmap_image, copy_text, get_clipboard_types, get_text};

mod common;
use common::{create_test_png, lock_clipboard};

#[test]
fn hc_clear_empties_clipboard_text() {
    let _guard = lock_clipboard();

    clear_clipboard().expect("clear_clipboard should work");
    copy_text("abc").expect("copy_text should work");

    let before = get_text().expect("get_text should work");
    assert_eq!(before, Some("abc".to_string()));

    clear_clipboard().expect("clear_clipboard should work");
    let after = get_text().expect("get_text should work after clear");
    assert_eq!(after, None);
}

#[test]
fn hc_text_sets_public_utf8_plain_text_type() {
    let _guard = lock_clipboard();

    copy_text("hello utf8").expect("copy_text should succeed");

    let types = get_clipboard_types().expect("get_clipboard_types should work");

    assert!(
        types.iter().flatten().any(|t| {
            let tl = t.to_lowercase();
            tl.contains("public.utf8-plain-text") || tl.contains("public.utf8")
        }),
        "expected clipboard types to include UTF-8 plain text, got: {:?}",
        types
    );
}

#[test]
fn hc_bitmap_sets_png_type_for_png() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let png = create_test_png(&tmp.path().join("img.png"));

    copy_bitmap_image(png.to_string_lossy().as_ref()).expect("copy_bitmap_image should succeed");

    let types = get_clipboard_types().expect("get_clipboard_types should work");

    assert!(
        types.iter().flatten().any(|t| {
            let tl = t.to_lowercase();
            tl.contains("png") || tl.contains("public.png")
        }),
        "expected clipboard types to include PNG for bitmap copy, got: {:?}",
        types
    );
}
