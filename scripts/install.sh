#!/bin/sh
set -e

# Configuration
REPO="bengaudry/pronto"
BINARY_NAME="pronto"

echo "Checking system compatibility..."
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map OS and Architecture to your GitHub Release asset names
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

# Construct the expected binary asset name (e.g., pronto-x86_64-apple-darwin)
ASSET_NAME="${BINARY_NAME}-${TARGET_ARCH}-${TARGET_OS}"
URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

echo "Downloading ${BINARY_NAME} from ${URL}..."

# Download to a temporary location
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

if curl -sSfL "$URL" -o "$BINARY_NAME"; then
    chmod +x "$BINARY_NAME"
else
    echo "Error: Failed to download the binary. Make sure the release asset exists."
    exit 1
fi

# Move to target binary directory
INSTALL_DIR="/usr/local/bin"
echo "Installing to ${INSTALL_DIR}/${BINARY_NAME} (may require sudo)..."

if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
else
    sudo mv "$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
fi

# Clean up
cd - > /dev/null
rm -rf "$TMP_DIR"

echo "Successfully installed ${BINARY_NAME}!"
echo "Run '${BINARY_NAME} --help' to verify the installation."
