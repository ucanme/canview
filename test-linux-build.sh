#!/bin/bash
# Test Linux build using Docker to simulate Ubuntu CI environment

set -e

echo "=== Testing Linux Build in Docker ==="

# Run Ubuntu container with Rust and dependencies
docker run --rm -v "$(pwd)":/app -w /app \
    -e CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
    ubuntu:22.04 \
    bash -c '
        echo "=== Installing dependencies ==="
        apt-get update
        apt-get install -y curl build-essential pkg-config \
            libxkbcommon-dev libx11-dev libegl1-mesa-dev \
            libfontconfig1-dev libfreetype6-dev \
            libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

        echo "=== Installing Rust ==="
        curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env

        echo "=== Rust version ==="
        rustc --version
        cargo --version

        echo "=== Building project for Linux ==="
        cargo build --release -p view

        echo "=== Build successful! ==="
        echo "Binary location: target/release/view"

        echo "=== Checking binary dependencies ==="
        ldd target/release/view
    '

echo "=== Linux build test completed successfully! ==="
