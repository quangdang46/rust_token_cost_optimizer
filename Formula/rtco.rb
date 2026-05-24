# typed: false
# frozen_string_literal: true

# Homebrew formula for rtco - Rust Token Killer
# To install: brew tap rtco-ai/tap && brew install rtco
class Rtk < Formula
  desc "High-performance CLI proxy to minimize LLM token consumption"
  homepage "https://www.rtco-ai.app"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/rtco-ai/rtco/releases/download/v#{version}/rtco-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_INTEL"
    end

    on_arm do
      url "https://github.com/rtco-ai/rtco/releases/download/v#{version}/rtco-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/rtco-ai/rtco/releases/download/v#{version}/rtco-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_INTEL"
    end

    on_arm do
      url "https://github.com/rtco-ai/rtco/releases/download/v#{version}/rtco-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM"
    end
  end

  def install
    bin.install "rtco"
  end

  test do
    assert_match "rtco #{version}", shell_output("#{bin}/rtco --version")
  end
end
