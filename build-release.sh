#!/bin/bash
set -e

echo "Building release binaries for all platforms..."
echo ""

# Build Windows
echo "Building Windows (x86_64-pc-windows-msvc)..."
cargo build --release --target x86_64-pc-windows-msvc || echo "Windows build failed (may need cross-compilation setup)"

# Build Linux
echo "Building Linux (x86_64-unknown-linux-gnu)..."
cargo build --release --target x86_64-unknown-linux-gnu || echo "Linux build failed (may need cross-compilation setup)"

# Build macOS Intel
echo "Building macOS Intel (x86_64-apple-darwin)..."
cargo build --release --target x86_64-apple-darwin || echo "macOS Intel build failed (may need cross-compilation setup)"

# Build macOS Apple Silicon
echo "Building macOS Apple Silicon (aarch64-apple-darwin)..."
cargo build --release --target aarch64-apple-darwin || echo "macOS Apple Silicon build failed (may need cross-compilation setup)"

echo ""
echo "Creating archives..."

# Create archives
if [ -f "target/x86_64-pc-windows-msvc/release/cli-t.exe" ]; then
    echo "Creating Windows archive..."
    cd target/x86_64-pc-windows-msvc/release
    zip -q ../../../cli-t-x86_64-pc-windows-msvc.zip cli-t.exe
    cd ../../..
fi

if [ -f "target/x86_64-unknown-linux-gnu/release/cli-t" ]; then
    echo "Creating Linux archive..."
    tar -czf cli-t-x86_64-unknown-linux-gnu.tar.gz -C target/x86_64-unknown-linux-gnu/release cli-t
fi

if [ -f "target/x86_64-apple-darwin/release/cli-t" ]; then
    echo "Creating macOS Intel archive..."
    tar -czf cli-t-x86_64-apple-darwin.tar.gz -C target/x86_64-apple-darwin/release cli-t
fi

if [ -f "target/aarch64-apple-darwin/release/cli-t" ]; then
    echo "Creating macOS Apple Silicon archive..."
    tar -czf cli-t-aarch64-apple-darwin.tar.gz -C target/aarch64-apple-darwin/release cli-t
fi

echo ""
echo "Done! Archives created in project root."

