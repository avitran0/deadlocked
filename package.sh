#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="${ROOT_DIR}/package"

cd "$ROOT_DIR"
mkdir -p "$OUTPUT_DIR"

build_and_stage() {
    local output_name="$1"
    shift

    cross build \
        --release \
        --target "x86_64-unknown-linux-gnu" \
        --bin deadlocked \
        "$@"

    local binary="${ROOT_DIR}/target/x86_64-unknown-linux-gnu/release/deadlocked"
    if [[ ! -f "$binary" ]]; then
        echo "error: expected build output not found: $binary" >&2
        exit 1
    fi

    cp "$binary" "${OUTPUT_DIR}/${output_name}"
    echo "generated ${OUTPUT_DIR}/${output_name}"
}

build_and_stage deadlocked
build_and_stage deadlocked-reduced-models --features reduced-models
