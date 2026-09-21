use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use hot_clipboard::{get_clipboard_types, get_file_urls, get_image_bytes, get_text};

use hot_clipboard::hp_ops::{
    copy_file, decode_clipboard_image, default_clip_ts_png, encode_image, ensure_dir,
    file_destination_for_single,
};

#[derive(Parser)]
#[command(name = "hp")]
#[command(about = "Hot Paste - Paste clipboard content to files or stdout", long_about = None)]
struct Cli {
    /// Output filename or directory
    output: Option<String>,

    /// Force overwrite existing files
    #[arg(short = 'f', long = "force")]
    force: bool,

    /// Specify output directory (created if missing)
    #[arg(short = 'd', long = "dir")]
    dir: Option<String>,

    /// Rename (force treating output as a filename even if it exists as a directory)
    #[arg(short = 'r', long = "rename")]
    rename: bool,

    /// Show clipboard info/metadata
    #[arg(short = 'i', long = "info")]
    info: bool,
}

fn main() {
    let args = Cli::parse();

    if args.info {
        let summary = hot_clipboard::inspect_clipboard_summary().unwrap_or_else(|_| {
            hot_clipboard::ClipboardSummary {
                kind: hot_clipboard::ClipboardKind::Empty,
                element_count: 0,
                approximate_bytes: 0,
                types: Vec::new(),
                file_paths: Vec::new(),
            }
        });

        println!("Clipboard kind: {:?}", summary.kind);
        println!("Clipboard types: {}", summary.types.join(", "));
        println!("Elements: {}", summary.element_count);
        println!("Approximate bytes: {}", summary.approximate_bytes);

        if !summary.file_paths.is_empty() {
            println!("File paths:");
            for p in summary.file_paths {
                println!("- {}", p);
            }
        }

        return;
    }

    // Priority 1: Finder files
    if let Ok(Some(file_paths)) = get_file_urls() {
        let src_paths: Vec<PathBuf> = file_paths.iter().map(PathBuf::from).collect();
        let n = src_paths.len();

        let explicit_dir = args.dir.as_deref();
        let output_str = args.output.as_deref();

        // Batch extension rename: hp -d <dir> "*.ext" [--force]
        if let Some(pattern) = output_str {
            if let Some(ext) = pattern.strip_prefix("*.") {
                if !ext.is_empty() {
                    let dest_dir = explicit_dir
                        .map(Path::new)
                        .unwrap_or_else(|| Path::new("."));
                    if let Err(e) = ensure_dir(dest_dir) {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }

                    use hot_clipboard::image_ops::convert_or_copy_file;

                    for src in &src_paths {
                        let filename = src.file_name().and_then(|s| s.to_str()).unwrap_or("file");
                        let stem = src
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| filename.to_string());

                        let dest = dest_dir.join(format!("{}.{}", stem, ext));

                        if dest.exists() && !args.force {
                            eprintln!(
                                "Error: File '{}' already exists. Use --force to overwrite.",
                                dest.display()
                            );
                            std::process::exit(1);
                        }

                        let bytes = match convert_or_copy_file(src, &dest, ext) {
                            Ok(b) => b,
                            Err(e) => {
                                eprintln!("{}", e);
                                std::process::exit(1);
                            }
                        };

                        if let Err(e) = fs::write(&dest, &bytes) {
                            eprintln!("Error: Failed to write file '{}': {}", dest.display(), e);
                            std::process::exit(1);
                        }
                    }
                    return;
                }
            }
        }

        // Destination directory for multi-file cases
        if n >= 2 {
            let dest_dir: &Path = if let Some(explicit) = explicit_dir {
                Path::new(explicit)
            } else if let Some(out) = output_str {
                // SPEC A.3: if the positional argument exists as a directory, paste into it.
                // But `-r/--rename` forces treating it as a filename (even if it's a directory),
                // so we still require --dir for multi-file when rename is set.
                let p = Path::new(out);
                if p.is_dir() && !args.rename {
                    p
                } else {
                    eprintln!(
                        "Error: Clipboard contains {} files. Specify a directory: hp -d <dir>",
                        n
                    );
                    std::process::exit(1);
                }
            } else {
                eprintln!(
                    "Error: Clipboard contains {} files. Specify a directory: hp -d <dir>",
                    n
                );
                std::process::exit(1);
            };

            if let Err(e) = ensure_dir(dest_dir) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            for src in &src_paths {
                let filename = src.file_name().unwrap();
                if let Err(e) = copy_file(src, &dest_dir.join(filename), args.force) {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
            return;
        }

        // If multiple files and no directory provided, the code above exits.

        // Single file case
        let src = &src_paths[0];

        // If --dir is provided, copy into that directory (ignore positional output unless --rename is set).
        if let Some(d) = explicit_dir {
            let dest_dir = Path::new(d);
            if let Err(e) = ensure_dir(dest_dir) {
                eprintln!("{}", e);
                std::process::exit(1);
            }

            if args.rename {
                if let Some(out) = output_str {
                    let rel_dest = match file_destination_for_single(src, out, true) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    };
                    let dest_name = rel_dest
                        .file_name()
                        .ok_or_else(|| "Output destination has no filename".to_string())
                        .unwrap();
                    let dest = dest_dir.join(dest_name);
                    if let Err(e) = copy_file(src, &dest, args.force) {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                    return;
                }
            }

            // Default: preserve original filename inside --dir.
            let filename = src.file_name().unwrap();
            if let Err(e) = copy_file(src, &dest_dir.join(filename), args.force) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            return;
        }

        // No --dir provided
        match output_str {
            None => {
                let filename = src.file_name().unwrap();
                let dest = Path::new(".").join(filename);
                if let Err(e) = copy_file(src, &dest, args.force) {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
            Some(out) => {
                let dest = match file_destination_for_single(src, out, args.rename) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                };
                if let Err(e) = copy_file(src, &dest, args.force) {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }

        return;
    }

    // Priority 2: Clipboard image (raw TIFF/PNG)
    if let Ok(Some((bytes, _kind))) = get_image_bytes() {
        let img = decode_clipboard_image(&bytes);
        let dest_dir = args
            .dir
            .as_deref()
            .map(Path::new)
            .unwrap_or_else(|| Path::new("."));
        if let Err(e) = ensure_dir(dest_dir) {
            eprintln!("{}", e);
            std::process::exit(1);
        }

        let target_path: PathBuf = if let Some(out) = args.output.as_deref() {
            let out_path = Path::new(out);
            let mut dest = out_path.to_path_buf();
            if dest.is_relative() {
                dest = dest_dir.join(dest);
            }
            if dest.extension().is_none() {
                dest.set_extension("png");
            }
            dest
        } else {
            dest_dir.join(default_clip_ts_png())
        };

        let ext = target_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png")
            .to_ascii_lowercase();

        if target_path.exists() && !args.force {
            eprintln!(
                "Error: File '{}' already exists. Use --force to overwrite.",
                target_path.display()
            );
            std::process::exit(1);
        }

        let out_bytes = encode_image(img, &ext);
        if let Err(e) = fs::write(&target_path, &out_bytes) {
            eprintln!(
                "Error: Failed to write file '{}': {}",
                target_path.display(),
                e
            );
            std::process::exit(1);
        }

        println!("✓ Pasted {}", target_path.display());
        return;
    }

    // Priority 3: Text
    match get_text() {
        Ok(Some(text)) => {
            if let Some(output_path) = args.output {
                // SPEC: respect --force for text output.
                if Path::new(&output_path).exists() && !args.force {
                    eprintln!(
                        "Error: File '{}' already exists. Use --force to overwrite.",
                        output_path
                    );
                    std::process::exit(1);
                }

                if let Err(e) = fs::write(&output_path, &text) {
                    eprintln!("Error: Failed to write file '{}': {}", output_path, e);
                    std::process::exit(1);
                }
                println!("✓ Pasted {} bytes to {}", text.len(), output_path);
            } else {
                print!("{}", text);
                io::stdout().flush().unwrap();
            }
        }
        _ => {
            // Spec: differentiate empty vs unknown binary.
            let types = get_clipboard_types().ok().flatten();
            if let Some(t) = types {
                if !t.is_empty() {
                    eprintln!(
                        "Error: Clipboard contains binary data. Specify a filename: hp <name>"
                    );
                    std::process::exit(1);
                }
            }

            eprintln!("Error: Clipboard is empty.");
            std::process::exit(1);
        }
    }
}
