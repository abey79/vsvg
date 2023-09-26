install:
    cargo install --path vsvg

clippy $RUSTFLAGS="-Dwarnings":
    cargo clippy --workspace --bins --examples

clippy-wasm $RUSTFLAGS="-Dwarnings":
    cargo clippy --workspace --exclude vsvg-multi --exclude vsvg-cli --bins --target wasm32-unknown-unknown

fmt:
    cargo fmt --all -- --check

web-build:
    cargo build -p whiskers-web-demo --lib --target wasm32-unknown-unknown --release
    wasm-bindgen target/wasm32-unknown-unknown/release/whiskers_web_demo.wasm --out-dir whiskers-web-demo/web --out-name whiskers_web_demo --no-modules --no-typescript

web-build-opt: web-build
    wasm-opt -Os whiskers-web-demo/web/whiskers_web_demo_bg.wasm -o whiskers-web-demo/web/whiskers_web_demo_bg.wasm

web-serve:
    basic-http-server whiskers-web-demo/web

test:
    cargo test
