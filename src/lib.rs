pub mod pasteboard;

pub mod hp_ops;
pub mod image_ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardKind {
    Files,
    Image,
    Text,
    Binary,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardSummary {
    pub kind: ClipboardKind,
    pub element_count: usize,
    pub approximate_bytes: usize,
    pub types: Vec<String>,
    pub file_paths: Vec<String>,
}

pub use pasteboard::{
    clear_clipboard, copy_bitmap_image, copy_files, copy_raw_bytes, copy_text, get_clipboard_size,
    get_clipboard_types, get_file_urls, get_image_bytes, get_text, inspect_clipboard,
    inspect_clipboard_summary,
};
