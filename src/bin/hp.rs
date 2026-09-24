use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use hot_clipboard::{get_clipboard_types, get_file_urls, get_image_bytes, get_text};

use hot_clipboard::hp_ops::{
    batch_extension_from_pattern, copy_file, decode_clipboard_image, default_clip_ts_png,
    destination_for_multiple_files, destination_for_single_file, encode_image, ensure_dir,
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
        let summary = hot_clipboard::inspect_clipboard_summary().unwrap_or_else(|error| {
            eprintln!("Error: {}", error);
            std::process::exit(1);
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
            if let Some(ext) = batch_extension_from_pattern(pattern) {
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

        // Destination directory for multi-file cases
        if n >= 2 {
            let dest_dir = destination_for_multiple_files(output_str, explicit_dir, args.rename, n)
                .unwrap_or_else(|error| {
                    eprintln!("{}", error);
                    std::process::exit(1);
                });

            if let Err(e) = ensure_dir(&dest_dir) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            for src in &src_paths {
                let filename = src.file_name().unwrap_or_else(|| {
                    eprintln!("Error: Source path has no filename: '{}'.", src.display());
                    std::process::exit(1);
                });
                if let Err(e) = copy_file(src, &dest_dir.join(filename), args.force) {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
            return;
        }

        // Single file case
        let src = &src_paths[0];
        let dest = destination_for_single_file(src, output_str, explicit_dir, args.rename)
            .unwrap_or_else(|error| {
                eprintln!("Error: {}", error);
                std::process::exit(1);
            });
        if let Some(explicit_dir) = explicit_dir {
            if let Err(error) = ensure_dir(Path::new(explicit_dir)) {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        }
        if let Err(error) = copy_file(src, &dest, args.force) {
            eprintln!("{}", error);
            std::process::exit(1);
        }

        return;
    }

    // Priority 2: Clipboard image (raw TIFF/PNG)
    if let Ok(Some((bytes, _kind))) = get_image_bytes() {
        let img = decode_clipboard_image(&bytes).unwrap_or_else(|error| {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        });
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

        let out_bytes = encode_image(img, &ext).unwrap_or_else(|error| {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        });
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
                if let Err(error) = io::stdout().flush() {
                    eprintln!("Error: Failed to flush stdout: {}", error);
                    std::process::exit(1);
                }
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
