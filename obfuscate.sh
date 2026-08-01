#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(pwd)"
OBFUSCATED_DIR="$PROJECT_DIR/target/obfuscated-source"

# Install obfuscator_cli if it isn't already installed
if ! command -v obfuscator_cli >/dev/null 2>&1; then
    echo "obfuscator_cli not found. Installing..."

    cargo install \
        --locked \
        --git https://github.com/GianIac/rustfuscator.git \
        --package obfuscator_cli

    echo "Installation complete."
fi

rm -rf "$OBFUSCATED_DIR"

obfuscator_cli \
    --input "$PROJECT_DIR" \
    --output "$OBFUSCATED_DIR" \
    --as-project

cd "$OBFUSCATED_DIR"
cargo build --release

echo
echo "Binary: $OBFUSCATED_DIR/target/release/tester"
