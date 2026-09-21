use std::process::Command;

mod common;
use common::{create_test_png, lock_clipboard};

use hot_clipboard::copy_bitmap_image;

#[test]
fn hp_image_without_args_uses_spec_timestamp() {
    let _guard = lock_clipboard();

    let tmp = tempfile::tempdir().unwrap();
    let img = create_test_png(&tmp.path().join("img.png"));
    copy_bitmap_image(img.to_string_lossy().as_ref()).expect("copy_bitmap_image");

    let hp_exe = env!("CARGO_BIN_EXE_hp");
    let status = Command::new(hp_exe).current_dir(&tmp).status().unwrap();
    assert!(status.success());

    let names: Vec<String> = std::fs::read_dir(&tmp)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.starts_with("clip_") && n.ends_with(".png"))
        .collect();

    assert_eq!(names.len(), 1, "expected one clip_*.png, got {names:?}");
    // SPEC: clip_YYYY-MM-DD_HHmmss.png
    let name = &names[0];
    let rest = name
        .strip_prefix("clip_")
        .unwrap()
        .strip_suffix(".png")
        .unwrap();
    let parts: Vec<&str> = rest.split('_').collect();
    assert_eq!(parts.len(), 2, "expected YYYY-MM-DD_HHmmss, got {name}");
    assert_eq!(parts[0].len(), 10, "date part: {name}");
    assert_eq!(parts[1].len(), 6, "time part: {name}");
    assert!(parts[0].chars().all(|c| c.is_ascii_digit() || c == '-'));
    assert!(parts[1].chars().all(|c| c.is_ascii_digit()));
}
