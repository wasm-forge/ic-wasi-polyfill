#!/bin/sh

set -e

##################################
############# prepare VARs

SDK_VERSION=27

SDK_DIR="$HOME/.cache/wasi-sdk"
OS=`uname -s`
ARCH=`uname -m`

# Normalize OS names
if [ "$OS" = "Darwin" ]; then
    OS="macos"
elif [ "$OS" = "Linux" ]; then
    OS="linux"
else
    echo "Unsupported OS: $OS"
    exit 1
fi

# Normalize architecture names
if [ "$ARCH" = "x86_64" ]; then
    ARCH="x86_64"
elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
    ARCH="arm64"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi


export WASI_DIR=wasi-sdk-$SDK_VERSION.0-$ARCH-$OS
export WASI_SDK_PATH=$SDK_DIR/$WASI_DIR

if [ "$1" = "--sdk" ]; then
  echo $WASI_SDK_PATH
  exit 0
fi

echo "Checking OS/Architecture combination: $OS-$ARCH"

export SRC=https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$SDK_VERSION/$WASI_DIR.tar.gz

if { [ "$OS" = "linux" ] && [ "$ARCH" = "x86_64" ]; } ||
   { [ "$OS" = "linux" ] && [ "$ARCH" = "arm64" ]; } ||
   { [ "$OS" = "macos" ] && [ "$ARCH" = "x86_64" ]; } ||
   { [ "$OS" = "macos" ] && [ "$ARCH" = "arm64" ]; }; then
    echo "✅ Detected supported platform: $OS-$ARCH"
else
    echo "❌ Unsupported OS/Architecture combination: $OS-$ARCH"
    exit 1
fi

##################################
############# download WASI-SDK

if [ ! -d "$WASI_SDK_PATH" ]; then

    echo "Downloading WASI-SDK..."
    
    mkdir -p "$SDK_DIR"

    curl -L -o "$SDK_DIR/wasi-sdk.tar.gz" "$SRC"

    echo "Extracting tar..."

    tar -xzf "$SDK_DIR/wasi-sdk.tar.gz" -C "$SDK_DIR"

    echo "Deleting download..."

    [ -f "$SDK_DIR/wasi-sdk.tar.gz" ] && rm "$SDK_DIR/wasi-sdk.tar.gz"

    echo "✅ WASI-SDK installed in: $WASI_SDK_PATH ..."
else
    echo "✅ WASI-SDK found in: $WASI_SDK_PATH ..."
fi


##################################
############# Update .bashrc

BASHRC="$HOME/.bashrc"

echo "Preparing .bashrc update..."

line1="export WASI_SDK_PATH=$WASI_SDK_PATH"
line2='export PATH=$WASI_SDK_PATH/bin:$PATH'

FOUND1=`grep -F "$line1" "$BASHRC" 2>/dev/null || true`
FOUND2=`grep -F "$line2" "$BASHRC" 2>/dev/null || true`

if [ -n "$FOUND1" ] && [ -n "$FOUND2" ]; then
    echo "✅ .bashrc is ready"
    exit 0
fi

AUTO_CONFIRM=false
if [ "$1" = "-y" ] || [ "$1" = "--yes" ]; then
  AUTO_CONFIRM=true
fi

if [ "$AUTO_CONFIRM" = "true" ]; then
  RESPONSE="Y"
else
  printf "Do you want to update yor .bashrc? [y/N] " 
  read RESPONSE
fi


case "$RESPONSE" in
  y|Y)
    echo "$line1" >> "$BASHRC"
    echo "$line2" >> "$BASHRC"
    echo "" >> "$BASHRC"
    echo "✅ .bashrc updated"
    echo "Restart your shell for the changes to take effect..."
    ;;
  *)
    echo "ℹ️ Skipped modifying .bashrc"
    echo 'To enable compilation, make sure you point $WASI_SDK_PATH to the WASI-SDK installation and ensure the WASI-oriented clang compiler is available on the PATH:'
    echo "$line1"
    echo "$line2"
    ;;
esac

