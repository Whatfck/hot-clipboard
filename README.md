# Hot CLIpboard

A modern, ergonomic take on `pbcopy`/`pbpaste` for macOS. `hc` copies files, text, and images; `hp` pastes, renames, and converts — all from the terminal.

**hot-clipboard** makes copy/paste between your terminal and macOS apps feel fast and ergonomic.

- **`hc` (Hot Copy):** copy files / text / images to the macOS clipboard
- **`hp` (Hot Paste):** paste clipboard content to files or stdout

Built on **`NSPasteboard`**.

Image conversion currently supports PNG, JPEG, and TIFF. WebP, GIF, and BMP are
not conversion targets.

---

<div align="center">
  <h3>Quick links</h3>
  <p>
    <a href="#usage">Usage</a> —
    <a href="#hc">hc</a> —
    <a href="#hp">hp</a> —
    <a href="#installation">Build/Install</a> —
    <a href="#testing">Testing</a>
  </p>
</div>

---

## Usage

### Copy files
```bash
hc ~/Documents/file1.txt ~/Pictures/photo.jpg
```

### Copy text from stdin (pipe)
```bash
cat notes.md | hc
# or
echo "hello" | hc
```

### Copy an image as a “bitmap” (inline image bytes)
```bash
hc -b photo.png
```

### Paste (auto by clipboard priority)
```bash
hp
```
Behavior depends on what’s in the clipboard:
1. **Finder files** → copies into the current directory
2. **Raw image** → writes `clip_<timestamp>.png`
3. **Text** → prints to stdout
4. **Unknown binary** → error (needs a filename)

---

## `hc` (Hot Copy)

### Flags
- `-b, --bitmap`  
  Copy an image file as bitmap (clipboard data bytes).
- `-c, --clear`  
  Clear the clipboard.

### Examples
Copy multiple files:
```bash
hc report.pdf notes.txt
```

Copy text:
```bash
git status | hc
```

Copy an image as bitmap:
```bash
hc -b icon.jpg
```

Clear clipboard:
```bash
hc -c
```

---

## `hp` (Hot Paste)

### Flags
- `-f, --force`  
  Overwrite destination if it exists.
- `-d, --dir <dir>`  
  Output directory (created if missing).
- `-r, --rename`  
  Treat positional `output` as a rename target (even if it matches an existing directory).
- `-i, --info`  
  Show clipboard metadata/types and detected payload summary.

### Examples

Paste Finder files into the current directory (default behavior for 1+ files):
```bash
hp
```

Paste into a directory:
```bash
hp -d ./Downloads
```

Rename a single file:
```bash
hp renamed.pdf
# if output has no extension, extension is inherited from the source file
```

Batch rename/conversion via pattern (when clipboard contains Finder files):
```bash
hp -d ./converted "*.jpg" --force
```

Paste text to a file:
```bash
hp > output.txt
```

---

## Installation

### Homebrew (via custom Tap)

To install `hot-clipboard`:

```bash
brew trust Whatfck/hot-clipboard
brew tap Whatfck/hot-clipboard
brew install hot-clipboard
```

Or in one line:

```bash
brew install Whatfck/hot-clipboard/hot-clipboard
```

> `brew trust` is required once: Homebrew refuses to load formulae from third-party taps until you explicitly trust them.

This installs from the [`Whatfck/homebrew-hot-clipboard`](https://github.com/Whatfck/homebrew-hot-clipboard) Tap. **Once approved in Homebrew Core**, you will be able to install with just `brew install hot-clipboard`.

### Build from source (Cargo)

macOS required (clipboard is macOS `NSPasteboard`).

```bash
git clone https://github.com/Whatfck/hot-clipboard.git
cd hot-clipboard
cargo build --release
```

Binaries:
- `./target/release/hc`
- `./target/release/hp`

---

## Testing

Run the full test suite:
```bash
cargo test --all
```

Tests interact with the macOS clipboard, so:
- they may require macOS privacy/access permissions
- clipboard access is protected by a test lock to reduce cross-test races

## Platform notes

`hot-clipboard` requires macOS because it uses `NSPasteboard`. Terminal and
the calling application may need permission to access the clipboard. The tool
supports PNG, JPEG, and TIFF for image conversion; WebP, GIF, and BMP are not
conversion targets. File URLs, UTF-8 text, and clipboard image data are read
through the standard macOS pasteboard types.

---

## Author
@Whatfck
