#!/usr/bin/env bash

# Installation script for the berry CLI tool

set -e

# Print the berry ASCII art
cat << "EOF"
    ____                        
   / __ )___  ____________  __
  / __  / _ \/ ___/ ___/ / / /
 / /_/ /  __/ /  / /  / /_/ / 
/_____/\___/_/  /_/   \__, /  
                     /____/    
EOF

echo "Installing berry CLI..."

# Ensure running on macOS
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

INSTALL_PATH="$INSTALL_DIR/berry"
VERSION="0.1.0"  # This should match your Cargo.toml version

# Try downloading pre-compiled binary first
BINARY_URL="https://berry.sasha.computer/berry-${VERSION}-darwin-arm64"
echo "Downloading pre-compiled binary from $BINARY_URL..."
if curl -sL "$BINARY_URL" -o "$INSTALL_PATH.tmp"; then
  # Verify the binary
  chmod +x "$INSTALL_PATH.tmp"
  if "$INSTALL_PATH.tmp" --version > /dev/null 2>&1; then
    mv "$INSTALL_PATH.tmp" "$INSTALL_PATH"
    echo "✓ Pre-compiled binary installed successfully"
  else
    echo "Downloaded binary verification failed, falling back to building from source..."
    rm -f "$INSTALL_PATH.tmp"
    BUILD_FROM_SOURCE=1
  fi
else
  echo "Pre-compiled binary not available, falling back to building from source..."
  BUILD_FROM_SOURCE=1
fi

# Build from source if needed
if [ "${BUILD_FROM_SOURCE}" = "1" ]; then
  # Check if cargo is installed
  if ! command -v cargo &> /dev/null; then
    echo "Error: Rust's cargo is required but not found."
    echo "Please install Rust first: https://www.rust-lang.org/tools/install"
    exit 1
  fi

  # Clone the repository to a temporary directory
  TMP_DIR=$(mktemp -d)
  echo "Cloning berry repository..."
  git clone https://github.com/sashaaldrick/berry.git "$TMP_DIR" || {
    echo "Error: Failed to clone repository"
    rm -rf "$TMP_DIR"
    exit 1
  }

  # Build the binary
  echo "Building berry..."
  (cd "$TMP_DIR" && cargo build --release) || {
    echo "Error: Failed to build berry"
    rm -rf "$TMP_DIR"
    exit 1
  }

  # Install the binary
  BINARY_PATH="$TMP_DIR/target/release/berry"
  echo "Installing berry to $INSTALL_PATH..."
  cp "$BINARY_PATH" "$INSTALL_PATH" || {
    echo "Error: Failed to install berry"
    rm -rf "$TMP_DIR"
    exit 1
  }

  chmod +x "$INSTALL_PATH"

  # Clean up
  rm -rf "$TMP_DIR"
  echo "✓ Built and installed from source successfully"
fi

# Determine the user's shell and corresponding config file
SHELL_CONFIG=""
case "$SHELL" in
  */zsh)
    SHELL_CONFIG="$HOME/.zshrc"
    ;;
  */bash)
    if [[ "$OSTYPE" == "darwin"* ]]; then
      SHELL_CONFIG="$HOME/.bash_profile"
    else
      SHELL_CONFIG="$HOME/.bashrc"
    fi
    ;;
  *)
    SHELL_CONFIG="$HOME/.profile"
    ;;
esac

# Add installation directory to PATH if needed
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo "Adding $INSTALL_DIR to your PATH in $SHELL_CONFIG"
  echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_CONFIG"
  echo "Please restart your terminal or run: source $SHELL_CONFIG"
fi

echo "✓ berry installed successfully!"
echo
echo "To get started, run:"
echo "  berry new my-project"
echo "  cd my-project"
echo "  berry setup"
echo
echo "For more information, visit: https://github.com/sashaaldrick/berry" 