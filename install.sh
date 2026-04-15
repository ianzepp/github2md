#!/usr/bin/env bash
set -euo pipefail

REPO="ianzepp/github2md"
BINARY="github2md"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

OS="$(uname -s)"
case "$OS" in
  Darwin) OS="apple-darwin" ;;
  Linux) OS="unknown-linux-gnu" ;;
  *)
    echo "Error: unsupported OS: $OS" >&2
    exit 1
    ;;
esac

ARCH="$(uname -m)"
case "$ARCH" in
  arm64|aarch64) ARCH="aarch64" ;;
  x86_64) ARCH="x86_64" ;;
  *)
    echo "Error: unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

TARGET="${ARCH}-${OS}"

if [[ "$TARGET" == "aarch64-unknown-linux-gnu" ]]; then
  echo "Error: no release artifact is currently published for ${TARGET}" >&2
  exit 1
fi

echo "Finding latest release..."
TAG="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)"
if [[ -z "$TAG" ]]; then
  echo "Error: could not find latest release" >&2
  exit 1
fi

echo "Latest release: ${TAG}"

ASSET="github2md-${TAG}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading ${ASSET}..."
curl -fsSL "$URL" -o "${TMPDIR}/${ASSET}"

tar xzf "${TMPDIR}/${ASSET}" -C "$TMPDIR"

if [[ -w "$INSTALL_DIR" ]]; then
  mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
  echo "Installing to ${INSTALL_DIR} (requires sudo)..."
  sudo mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi

chmod +x "${INSTALL_DIR}/${BINARY}"

echo "Installed ${BINARY} ${TAG} to ${INSTALL_DIR}/${BINARY}"
