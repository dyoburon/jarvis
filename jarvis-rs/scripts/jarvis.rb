# =============================================================================
# Homebrew formula for Jarvis
# =============================================================================
#
# Install: brew install --build-from-source jarvis.rb
# Or from tap: brew install dylan/tap/jarvis

class Jarvis < Formula
  desc "GPU-accelerated terminal emulator with AI integration"
  homepage "https://github.com/dylan/jarvis"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/dylan/jarvis/releases/download/v#{version}/jarvis-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    else
      url "https://github.com/dylan/jarvis/releases/download/v#{version}/jarvis-macos-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  on_linux do
    url "https://github.com/dylan/jarvis/releases/download/v#{version}/jarvis-linux-x86_64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX"
  end

  def install
    bin.install "jarvis"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/jarvis --version")
  end
end
