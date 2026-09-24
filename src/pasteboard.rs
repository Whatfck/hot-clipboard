use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, ProtocolObject};
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
            file_urls.push(file_url);
            filenames.push(path_string.to_string());
        }

        pasteboard.clearContents();

        let file_url_objects = file_urls
            .into_iter()
            .map(ProtocolObject::from_retained)
            .collect::<Vec<_>>();
        let file_url_array = NSArray::from_vec(file_url_objects);
        let public_url_written = pasteboard.writeObjects(&file_url_array);

        let path_strings: Vec<_> = filenames.iter().map(|s| NSString::from_str(s)).collect();
        let paths_array = NSArray::from_vec(path_strings);
        let type_string = NSString::from_str("NSFilenamesPboardType");
        // `paths_array` is NSArray<NSString>, which is the property-list shape
        // required by the legacy NSFilenamesPboardType contract.
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
        } else if let Some(ns_string) =
            pasteboard.stringForType(&NSString::from_str("public.utf8-plain-text"))
        {
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

        let url_class = {
            let class: *const AnyClass = NSURL::class();
            Retained::retain(class as *mut AnyObject)
                .ok_or_else(|| "Failed to access NSURL class".to_string())?
        };
        let class_array = NSArray::from_vec(vec![url_class]);
        if let Some(objects) = pasteboard.readObjectsForClasses_options(&class_array, None) {
            let mut paths = Vec::new();
            for index in 0..objects.len() {
                let object = objects.objectAtIndex(index);
                // readObjectsForClasses_options was restricted to NSURL above,
                // so each returned object is safe to cast to NSURL here.
                let url: Retained<NSURL> = Retained::cast(object);
                if let Some(path) = url.path() {
                    paths.push(path.to_string());
                }
            }
            if !paths.is_empty() {
                return Ok(Some(paths));
            }
        }

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
                        let value = ns_string.to_string();
                        if type_name.to_string() == "public.file-url"
                            || type_name.to_string() == "public.url"
                        {
                            if let Some(url) = NSURL::URLWithString(&ns_string) {
                                if let Some(path) = url.path() {
                                    paths.push(path.to_string());
                                }
                            }
                        } else {
                            paths.push(value);
                        }
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
    let types = get_clipboard_types()?.unwrap_or_default();
    let file_paths = get_file_urls()?.unwrap_or_default();
    let text = get_text()?;
    let image = get_image_bytes()?;

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
    let file_urls = get_file_urls()?;
    let text = get_text()?;
    let image = get_image_bytes()?;

    match () {
        _ if file_urls.as_ref().is_some_and(|files| !files.is_empty()) => {
            let total: usize = file_urls
                .as_ref()
                .into_iter()
                .flatten()
                .filter_map(|p| std::fs::metadata(p).ok())
                .map(|m| m.len() as usize)
                .sum();
            Ok(Some(total))
        }
        _ if let Some(text) = text.as_ref() => Ok(Some(text.len())),
        _ if let Some((data, _)) = image.as_ref() => Ok(Some(data.len())),
        _ => Ok(None),
    }
}

/// Copy image file as bitmap to clipboard.
pub fn copy_bitmap_image(path: &str) -> Result<(), String> {
    let source_bytes = std::fs::read(path).map_err(|e| format!("Failed to read image: {}", e))?;
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    unsafe {
        use objc2_foundation::NSData;

        let (bytes, type_to_set) = match ext.as_deref() {
            Some("png") => (source_bytes, objc2_app_kit::NSPasteboardTypePNG),
            _ => {
                let image = crate::image_ops::decode_image(&source_bytes)?;
                let bytes = crate::image_ops::encode_image(image, "tiff")?;
                (bytes, objc2_app_kit::NSPasteboardTypeTIFF)
            }
        };

        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();

        let data = NSData::with_bytes(&bytes);

        let ok = pasteboard.setData_forType(Some(&data), type_to_set);

        if ok {
            Ok(())
        } else {
            Err("Failed to write bitmap image to clipboard".to_string())
        }
    }
}

/// Write raw bytes under an arbitrary pasteboard type.
///
/// This is primarily useful for integration tests and diagnostics that need to
/// model an unknown pasteboard payload.
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
