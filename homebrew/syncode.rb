# Homebrew Formula for Syncode
# Usage: brew install Agions/tap/syncode
#
# To set up the tap (one-time):
#   brew tap Agions/tap https://gitee.com/Agions/homebrew-tap.git
#   brew install syncode

class Syncode < Formula
  desc "AI Pair Programming Terminal — 让 AI 与你的代码同步"
  homepage "https://gitee.com/Agions/syncode"
  url "https://gitee.com/Agions/syncode/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "" # Will be filled after release
  license "MIT"
  version "1.0.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    bin.install "target/release/syncode"
  end

  test do
    assert_match "syncode", shell_output("#{bin}/syncode --version")
  end
end
