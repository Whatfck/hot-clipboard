use objc2::runtime::AnyObject;
use objc2::ClassType;
use objc2_app_kit::NSPasteboard;
use objc2_foundation::{NSArray, NSString, NSURL};
use std::path::Path;

/// Clear macOS clipboard
/// This function clears the contents of the macOS clipboard.
/// It returns a Result indicating success or failure.
pub fn clear_clipboard() -> Result<(), String> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        Ok(())
    }
}

/// Copy plain text to macOS clipboard
/// This function takes a string and copies it to the clipboard.
/// Returns a Result indicating success or failure.
pub fn copy_text(text: &str) -> Result<(), String> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();

        let ns_string = NSString::from_str(text);
        let utf8_type = NSString::from_str("public.utf8-plain-text");
        let legacy_success =
            pasteboard.setString_forType(&ns_string, objc2_app_kit::NSPasteboardTypeString);
        let utf8_success = pasteboard.setString_forType(&ns_string, &utf8_type);

        if legacy_success || utf8_success {
            Ok(())
        } else {
            Err("Failed to write to pasteboard".to_string())
        }
    }
}

/// Copy files to clipboard as Finder file references
pub fn copy_files(paths: &[String]) -> Result<(), String> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();

        let mut file_urls = Vec::new();
        let mut filenames = Vec::new();

        for path_str in paths {
            let path = Path::new(path_str);

            let resolved = if path.is_absolute() {
                path.canonicalize()
                    .map_err(|_| format!("No such file or directory: '{}'", path_str))?
            } else {
                let joined = std::env::current_dir()
                    .map_err(|e| format!("Failed to get current directory: {}", e))?
                    .join(path);
                joined
                    .canonicalize()
                    .map_err(|_| format!("No such file or directory: '{}'", path_str))?
            };

            let path_string = resolved
                .to_str()
                .ok_or_else(|| format!("Invalid path: '{}'", path_str))?;

            let ns_path = NSString::from_str(path_string);
            let file_url = NSURL::fileURLWithPath(&ns_path);
            let file_url_string = file_url.absoluteString().unwrap_or_default();

            file_urls.push(file_url_string);
            filenames.push(path_string.to_string());
        }

        let file_url_array = NSArray::from_vec(file_urls);
        let public_url_type = NSString::from_str("public.file-url");
        let public_url_written = pasteboard
            .setPropertyList_forType((*file_url_array).as_super() as &AnyObject, &public_url_type);

        let path_strings: Vec<_> = filenames.iter().map(|s| NSString::from_str(s)).collect();
        let paths_array = NSArray::from_vec(path_strings);
        let type_string = NSString::from_str("NSFilenamesPboardType");
        let legacy_written = pasteboard
            .setPropertyList_forType((*paths_array).as_super() as &AnyObject, &type_string);

        if public_url_written || legacy_written {
            Ok(())
        } else {
            Err("Failed to write files to pasteboard".to_string())
        }
    }
}

/// Read plain text from macOS clipboard
pub fn get_text() -> Result<Option<String>, String> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();

        if let Some(ns_string) = pasteboard.stringForType(objc2_app_kit::NSPasteboardTypeString) {
            Ok(Some(ns_string.to_string()))
        } else {
            Ok(None)
        }
    }
}

/// Read file URLs from clipboard (files copied in Finder)
pub fn get_file_urls() -> Result<Option<Vec<String>>, String> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        let candidate_types = [
            NSString::from_str("NSFilenamesPboardType"),
            NSString::from_str("public.file-url"),
            NSString::from_str("public.url"),
        ];

        for type_name in candidate_types {
            if let Some(property_list) = pasteboard.propertyListForType(&type_name) {
                use objc2::msg_send_id;
                use objc2::rc::Retained;

                let array: Option<Retained<NSArray<NSString>>> = msg_send_id![&property_list, self];

                if let Some(array) = array {
                    let mut paths = Vec::new();
                    let count = array.len();

                    for i in 0..count {
                        let ns_string = array.objectAtIndex(i);
                        paths.push(ns_string.to_string());
                    }

                    if !paths.is_empty() {
                        return Ok(Some(paths));
                    }
                }
            }
        }

        Ok(None)
    }
}

/// Try to read image bytes from clipboard.
/// Returns raw encoded bytes plus the detected pasteboard type.
pub fn get_image_bytes() -> Result<Option<(Vec<u8>, String)>, String> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();

        // Prefer PNG first
        if let Some(data) = pasteboard.dataForType(objc2_app_kit::NSPasteboardTypePNG) {
            let bytes = data.bytes().to_vec();
            return Ok(Some((bytes, "png".to_string())));
        }

        // Then TIFF
        if let Some(data) = pasteboard.dataForType(objc2_app_kit::NSPasteboardTypeTIFF) {
            let bytes = data.bytes().to_vec();
            return Ok(Some((bytes, "tiff".to_string())));
        }

        Ok(None)
    }
}

/// List clipboard type strings (UTIs / pasteboard types).
pub fn get_clipboard_types() -> Result<Option<Vec<String>>, String> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        if let Some(types) = pasteboard.types() {
            let mut out = Vec::new();
            let count = types.len();
            for i in 0..count {
                let t = types.objectAtIndex(i);
                out.push(t.to_string());
            }
            Ok(Some(out))
        } else {
            Ok(None)
        }
    }
}

/// Inspect clipboard contents based on the project priority order.
pub fn inspect_clipboard() -> Result<crate::ClipboardKind, String> {
    Ok(inspect_clipboard_summary()?.kind)
}

pub fn inspect_clipboard_summary() -> Result<crate::ClipboardSummary, String> {
    let types = get_clipboard_types().ok().flatten().unwrap_or_default();
    let file_paths = get_file_urls().ok().flatten().unwrap_or_default();
    let text = get_text().ok().flatten();
    let image = get_image_bytes().ok().flatten();

    let kind = if !file_paths.is_empty() {
        crate::ClipboardKind::Files
    } else if image.is_some() {
        crate::ClipboardKind::Image
    } else if text.is_some() {
        crate::ClipboardKind::Text
    } else if !types.is_empty() {
        crate::ClipboardKind::Binary
    } else {
        crate::ClipboardKind::Empty
    };

    let element_count = if !file_paths.is_empty() {
        file_paths.len()
    } else if text.is_some() || image.is_some() {
        1
    } else {
        0
    };

    let approximate_bytes = if !file_paths.is_empty() {
        file_paths
            .iter()
            .filter_map(|p| std::fs::metadata(p).ok())
            .map(|m| m.len() as usize)
            .sum()
    } else if let Some(text) = text {
        text.len()
    } else if let Some((data, _)) = image {
        data.len()
    } else {
        0
    };

    Ok(crate::ClipboardSummary {
        kind,
        element_count,
        approximate_bytes,
        types: types.clone(),
        file_paths: file_paths.clone(),
    })
}

/// Approximate byte size of the current clipboard payload.
pub fn get_clipboard_size() -> Result<Option<usize>, String> {
    let file_urls = get_file_urls().ok().flatten();
    let text = get_text().ok().flatten();
    let image = get_image_bytes().ok().flatten();

    match () {
        _ if file_urls.as_ref().is_some_and(|files| !files.is_empty()) => {
            let total: usize = file_urls
                .unwrap()
                .iter()
                .filter_map(|p| std::fs::metadata(p).ok())
                .map(|m| m.len() as usize)
                .sum();
            Ok(Some(total))
        }
        _ if text.is_some() => Ok(Some(text.unwrap().len())),
        _ if image.is_some() => Ok(Some(image.unwrap().0.len())),
        _ => Ok(None),
    }
}

/// Copy image file as bitmap to clipboard.
/// For now this is implemented by reading the file bytes and writing them as TIFF data.
pub fn copy_bitmap_image(path: &str) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read image: {}", e))?;

    // Best-effort: choose a clipboard type based on file extension.
    // - .png   -> NSPasteboardTypePNG
    // - others -> NSPasteboardTypeTIFF (fallback)
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());

    unsafe {
        use objc2_foundation::NSData;

        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();

        let data = NSData::with_bytes(&bytes);

        let type_to_set = match ext.as_deref() {
            Some("png") => objc2_app_kit::NSPasteboardTypePNG,
            _ => objc2_app_kit::NSPasteboardTypeTIFF,
        };

        let ok = pasteboard.setData_forType(Some(&data), type_to_set);

        if ok {
            Ok(())
        } else {
            Err("Failed to write bitmap image to clipboard".to_string())
        }
    }
}

/// Write raw bytes under an arbitrary pasteboard type (tests / unknown binary).
pub fn copy_raw_bytes(bytes: &[u8], pasteboard_type: &str) -> Result<(), String> {
    unsafe {
        use objc2_foundation::NSData;

        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();

        let data = NSData::with_bytes(bytes);
        let type_string = NSString::from_str(pasteboard_type);
        let ok = pasteboard.setData_forType(Some(&data), &type_string);

        if ok {
            Ok(())
        } else {
            Err("Failed to write raw bytes to clipboard".to_string())
        }
    }
}
