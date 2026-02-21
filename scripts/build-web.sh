#!/usr/bin/env sh

GAME_NAME="hack-club-space-program"
OUT_DIR="./target/wasm32-unknown-unknown/web"
RELEASE_MODE=0

for arg in "$@"; do
    if [ "$arg" = "--release" ]; then
        RELEASE_MODE=1
    fi
done

WASM_PATH="./target/wasm32-unknown-unknown/release/$GAME_NAME.wasm"

if [ "$RELEASE_MODE" -eq 0 ]; then
    WASM_PATH="./target/wasm32-unknown-unknown/debug/$GAME_NAME.wasm"
fi

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

if [ "$RELEASE_MODE" -eq 0 ]; then
    cargo build --target wasm32-unknown-unknown
else
    cargo build --release --target wasm32-unknown-unknown
fi

mkdir -p "$OUT_DIR"
wasm-bindgen \
    --target web \
    --out-dir "$OUT_DIR" \
    --out-name "$GAME_NAME" \
    "$WASM_PATH"

if [ "$RELEASE_MODE" -ne 0 ]; then
    : # TODO: wasm-opt
fi

cp ./web/index.html "$OUT_DIR"