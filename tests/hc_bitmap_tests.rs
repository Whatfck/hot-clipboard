mod common;
use common::{create_test_jpg, lock_clipboard};
use image::ImageFormat;

use hot_clipboard::{copy_bitmap_image, get_clipboard_types, get_image_bytes};

#[test]
fn hc_bitmap_sets_tiff_type_for_jpg() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let jpg = create_test_jpg(&tmp.path().join("img.jpg"));

    copy_bitmap_image(jpg.to_string_lossy().as_ref()).expect("copy_bitmap_image should succeed");

    let types = get_clipboard_types().expect("get_clipboard_types should work");
    assert!(
        types
            .iter()
            .flatten()
            .any(|t| t.to_lowercase().contains("tiff")),
        "expected clipboard types to include TIFF for jpg bitmap copy, got: {:?}",
        types
    );

    let image_bytes = get_image_bytes().expect("get_image_bytes should work");
    let Some((bytes, kind)) = image_bytes else {
        panic!("expected clipboard to contain image bytes");
    };

    assert!(!bytes.is_empty());
    assert!(kind.to_lowercase().contains("tiff"));
    let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Tiff)
        .expect("bitmap bytes announced as TIFF must be valid TIFF data");
    assert_eq!((decoded.width(), decoded.height()), (2, 2));
}
