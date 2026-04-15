#!/usr/bin/env bash
set -euo pipefail

TAG=""
REPO=""
OUTPUT=""
DARWIN_ARM64_SHA=""
DARWIN_X86_64_SHA=""
LINUX_X86_64_SHA=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      TAG="$2"
      shift 2
      ;;
    --repo)
      REPO="$2"
      shift 2
      ;;
    --output)
      OUTPUT="$2"
      shift 2
      ;;
    --darwin-arm64-sha)
      DARWIN_ARM64_SHA="$2"
      shift 2
      ;;
    --darwin-x86_64-sha)
      DARWIN_X86_64_SHA="$2"
      shift 2
      ;;
    --linux-x86_64-sha)
      LINUX_X86_64_SHA="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$TAG" || -z "$REPO" || -z "$OUTPUT" || -z "$DARWIN_ARM64_SHA" || -z "$DARWIN_X86_64_SHA" || -z "$LINUX_X86_64_SHA" ]]; then
  echo "missing required arguments" >&2
  exit 1
fi

cat > "$OUTPUT" <<EOF
class Github2md < Formula
  desc "Export GitHub issues to Markdown files"
  homepage "https://github.com/${REPO}"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/${REPO}/releases/download/${TAG}/github2md-${TAG}-aarch64-apple-darwin.tar.gz"
    sha256 "${DARWIN_ARM64_SHA}"
  elsif OS.mac?
    url "https://github.com/${REPO}/releases/download/${TAG}/github2md-${TAG}-x86_64-apple-darwin.tar.gz"
    sha256 "${DARWIN_X86_64_SHA}"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/${REPO}/releases/download/${TAG}/github2md-${TAG}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "${LINUX_X86_64_SHA}"
  else
    odie "Unsupported platform"
  end

  def install
    bin.install "github2md"
  end

  test do
    assert_match "github2md", shell_output("#{bin}/github2md --help")
  end
end
EOF
