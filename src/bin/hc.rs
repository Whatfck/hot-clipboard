use clap::Parser;
use hot_clipboard::{clear_clipboard, copy_bitmap_image, copy_files, copy_text};
use std::io::{self, IsTerminal, Read};

#[derive(Parser)]
#[command(name = "hc")]
#[command(about = "Hot Copy - Copy files and text to the macOS clipboard", long_about = None)]
struct Cli {
    /// Files to copy to clipboard
    files: Vec<String>,
    /// Copy image as bitmap instead of file reference
    #[arg(short = 'b', long = "bitmap")]
    bitmap: bool,
    /// Clear the clipboard
    #[arg(short = 'c', long = "clear")]
    clear: bool,
}

fn main() {
    let args = Cli::parse();

    if args.clear {
        if let Err(e) = clear_clipboard() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        println!("✓ Cleared clipboard");
        return;
    }

    if args.bitmap {
        if args.files.len() != 1 {
            eprintln!("Error: Bitmap mode expects exactly 1 image file.");
            std::process::exit(1);
        }
        let path = &args.files[0];
        if let Err(e) = copy_bitmap_image(path) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        println!("✓ Copied image as bitmap to clipboard");
        return;
    }

    // Rule: file args have priority. Text is only accepted via stdin when no file args are provided.
    if !args.files.is_empty() {
        match copy_files(&args.files) {
            Ok(_) => println!("✓ Copied {} file(s) to clipboard", args.files.len()),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // No files: detect stdin (pipe)
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        let mut bytes = Vec::new();
        stdin
            .lock()
            .read_to_end(&mut bytes)
            .unwrap_or_else(|error| {
                eprintln!("Error: Failed to read stdin: {}", error);
                std::process::exit(1);
            });

        let buffer = String::from_utf8(bytes).unwrap_or_else(|_| {
            eprintln!("Error: stdin is not valid UTF-8.");
            std::process::exit(1);
        });

        match copy_text(&buffer) {
            Ok(_) => println!("✓ Copied {} bytes to clipboard", buffer.len()),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    eprintln!("Error: No files provided and no stdin detected.");
    std::process::exit(1);
}
