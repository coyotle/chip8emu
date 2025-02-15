#!/bin/bash

set -e

VERSION=$(grep -m1 'version' Cargo.toml | cut -d '"' -f 2)

ARTIFACTS_DIR="artifacts/$VERSION"

mkdir -p "$ARTIFACTS_DIR"

build_and_package() {
    local target=$1
    local binary_name=$2
    local archive_name=$3

    echo "Buildind for $target..."
    cargo build --release --target "$target"

    echo "Creating archive $binary_name..."
    if [[ "$target" == *"windows"* ]]; then
        zip -9 -j "$ARTIFACTS_DIR/$archive_name.zip" "target/$target/release/$binary_name.exe"
    else
        tar -czf "$ARTIFACTS_DIR/$archive_name.tar.gz" -C "target/$target/release/" "$binary_name"
    fi

    echo "Finished: $ARTIFACTS_DIR/$archive_name.*"
}

build_and_package "x86_64-unknown-linux-gnu" "chip8emu" "chip8emu-$VERSION-linux-x86_64"

build_and_package "x86_64-pc-windows-gnu" "chip8emu" "chip8emu-$VERSION-windows-x86_64"

echo "Artefacts placed in $ARTIFACTS_DIR"
