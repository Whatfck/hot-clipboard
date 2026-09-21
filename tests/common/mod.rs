use image::{ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static IN_PROCESS_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub struct TestClipboardGuard {
    _mem_guard: std::sync::MutexGuard<'static, ()>,
    file: File,
}

impl Drop for TestClipboardGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

/// Acquire both an in-process Mutex and a system-wide OS file lock
/// so that integration test binaries running concurrently never race on NSPasteboard.
pub fn lock_clipboard() -> TestClipboardGuard {
    let mem_guard = IN_PROCESS_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let lock_path = std::env::temp_dir().join("hot_clipboard_test.lock");
    let file = File::create(&lock_path).expect("Failed to open test clipboard lock file");
    file.lock()
        .expect("Failed to acquire OS lock on test clipboard lock file");

    TestClipboardGuard {
        _mem_guard: mem_guard,
        file,
    }
}

#[allow(dead_code)]
pub fn create_test_png(path: &Path) -> PathBuf {
    let img = RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
    img.save_with_format(path, ImageFormat::Png)
        .expect("Failed to create test PNG");
    path.to_path_buf()
}

#[allow(dead_code)]
pub fn create_test_jpg(path: &Path) -> PathBuf {
    // JPEG doesn't support alpha; convert to RGB.
    let rgb = RgbImage::from_pixel(2, 2, Rgb([0, 255, 0]));
    rgb.save_with_format(path, ImageFormat::Jpeg)
        .expect("Failed to create test JPEG");
    path.to_path_buf()
}
