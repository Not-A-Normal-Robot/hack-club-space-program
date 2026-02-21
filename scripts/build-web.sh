#!/usr/bin/env sh

GAME_NAME="hack-club-space-program"
OUT_DIR="./target/wasm32-unknown-unknown/web"

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo not installed and is required to build for web"
    echo "one way to get cargo and the rest of the rust toolchain is rustup: https://rustup.rs"
    exit 1
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
    echo "wasm-bindgen-cli not found, installing..."
    cargo install wasm-bindgen-cli
fi

rustup target add wasm32-unknown-unknown

cargo build --release --target wasm32-unknown-unknown

mkdir -p "$OUT_DIR"
wasm-bindgen --no-typescript \
    --target web \
    --out-dir "$OUT_DIR" \
    --out-name "$GAME_NAME" \
    "./target/wasm32-unknown-unknown/release/$GAME_NAME.wasm"

# TODO: wasm-opt

cp ./web/* "$OUT_DIR"