#!/usr/bin/env bash

# Minimal installer script for our CLI tool (emulating rustup behavior)

set -e

# Ensure script is running on macOS
if [[ "$(uname)" != "Darwin" ]]; then
  echo "Error: This installer currently only supports macOS."
  exit 1
fi

# Ensure running on Apple Silicon (arm64)
ARCH=$(uname -m)
if [[ "$ARCH" != "arm64" ]]; then
  echo "Error: This installer is designed for macOS Apple Silicon (arm64)."
  exit 1
fi

# Define URL for the pre-compiled CLI binary (placeholder URL)
BINARY_URL="https://example.com/cli-macos-arm64"

# Create a temporary file for downloading the binary
TMP_BINARY=$(mktemp)

echo "Downloading CLI binary from $BINARY_URL ..."
if ! curl -fsSL "$BINARY_URL" -o "$TMP_BINARY"; then
  echo "Error: Failed to download the CLI binary."
  exit 1
fi

# Make the downloaded binary executable
chmod +x "$TMP_BINARY"

# Determine the installation directory
# Default is /usr/local/bin. If not writable, try /opt/homebrew/bin (common on Apple Silicon), else fallback to $HOME/.local/bin
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
  if [ -d "/opt/homebrew/bin" ] && [ -w "/opt/homebrew/bin" ]; then
    INSTALL_DIR="/opt/homebrew/bin"
  else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
  fi
fi

# Define the CLI binary name (adjust as needed)
BIN_NAME="mycli"
INSTALL_PATH="$INSTALL_DIR/$BIN_NAME"

echo "Installing CLI binary to $INSTALL_PATH ..."
if ! mv "$TMP_BINARY" "$INSTALL_PATH"; then
  echo "Error: Could not move binary to $INSTALL_PATH."
  exit 1
fi

chmod +x "$INSTALL_PATH"

# Determine the user's shell and corresponding config file for modifying PATH
CURRENT_SHELL=$(basename "$SHELL")
case "$CURRENT_SHELL" in
  zsh)
    SHELL_CONFIG="$HOME/.zshrc"
    ;;
  bash)
    SHELL_CONFIG="$HOME/.bash_profile"
    ;;
  *)
    SHELL_CONFIG="$HOME/.profile"
    ;;
esac

echo "Ensuring $INSTALL_DIR is in your PATH in $SHELL_CONFIG ..."

# Check if the PATH update already exists
if ! grep -q "$INSTALL_DIR" "$SHELL_CONFIG"; then
  echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_CONFIG"
  echo "Added $INSTALL_DIR to PATH in $SHELL_CONFIG. Please restart your terminal or run 'source $SHELL_CONFIG'."
else
  echo "$INSTALL_DIR is already in PATH in $SHELL_CONFIG."
fi

# Placeholder for checking dependencies
echo "Checking for required dependencies... (this is a placeholder)"

# Future: Add actual dependency checks here

echo "Installation complete. You can now use the '$BIN_NAME' command." 