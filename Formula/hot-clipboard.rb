class HotClipboard < Formula
  desc "Ergonomic macOS clipboard CLI bridging terminal and NSPasteboard"
  homepage "https://github.com/Whatfck/hot-clipboard"
  url "https://github.com/Whatfck/hot-clipboard/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256_ONCE_TAGGED"
  license "MIT"
  head "https://github.com/Whatfck/hot-clipboard.git", branch: "develop"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "Hot Copy", shell_output("#{bin}/hc --help")
    assert_match "Hot Paste", shell_output("#{bin}/hp --help")
  end
end
