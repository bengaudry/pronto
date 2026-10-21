#!/bin/sh
set -e

# Configuration
REPO="bengaudry/pronto"
BINARY_NAME="pronto"

echo "Checking system compatibility..."
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    darwin)
        TARGET_OS="apple-darwin"
        ;;
    linux)
        TARGET_OS="unknown-linux-gnu"
        ;;
    *)
        echo "Error: Unsupported operating system: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        TARGET_ARCH="x86_64"
        ;;
    arm64|aarch64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

ASSET_NAME="${BINARY_NAME}-${TARGET_ARCH}-${TARGET_OS}"
URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

echo "Downloading ${BINARY_NAME} from ${URL}..."

TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

if curl -sSfL "$URL" -o "$BINARY_NAME"; then
    chmod +x "$BINARY_NAME"
else
    echo "Error: Failed to download the binary. Make sure the release asset exists."
    exit 1
fi

# Target user directory (no root required)
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

echo "Installing to ${INSTALL_DIR}/${BINARY_NAME}..."
mv "$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"

# Clean up
cd - > /dev/null
rm -rf "$TMP_DIR"

echo "Successfully installed ${BINARY_NAME}!"

# Warn if directory is not reachable via PATH
case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        echo "Notice: ${INSTALL_DIR} is not in your PATH."
        echo "Add it to your shell profile: export PATH=\"${INSTALL_DIR}:\$PATH\""
        ;;
esac

echo "Run '${BINARY_NAME} --help' to verify the installation."
