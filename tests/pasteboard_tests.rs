use hot_clipboard::{copy_text, get_text};

mod common;
use common::lock_clipboard;

#[test]
fn test_copy_text_succeeds() {
    let _guard = lock_clipboard();

    let text = "Hello, Clipboard!";
    copy_text(text).expect("copy_text should succeed");

    let got = get_text().expect("get_text should work");
    assert_eq!(
        got,
        Some(text.to_string()),
        "Text copied to clipboard should match"
    );
}

#[test]
fn test_copy_empty_string() {
    let _guard = lock_clipboard();

    copy_text("").expect("copy_text should work with empty string");

    let got = get_text().expect("get_text should work");
    assert_eq!(got, Some("".to_string()), "Empty string should be copied");
}

#[test]
fn test_copy_text_overwrites_previous_clipboard_content() {
    let _guard = lock_clipboard();

    copy_text("Old Content").expect("copy_text should succeed");
    assert_eq!(
        get_text().expect("get_text should work"),
        Some("Old Content".to_string())
    );

    copy_text("New Content").expect("copy_text should succeed");
    assert_eq!(
        get_text().expect("get_text should work"),
        Some("New Content".to_string())
    );
}
