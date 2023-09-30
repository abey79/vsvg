install:
    cargo install --path crates/vsvg-cli

clippy $RUSTFLAGS="-Dwarnings":
    cargo clippy --workspace --all-features --bins --examples

clippy-wasm $RUSTFLAGS="-Dwarnings":
    cargo clippy --workspace --all-features --exclude msvg --exclude vsvg-cli --bins --target wasm32-unknown-unknown

fmt:
    cargo fmt --all -- --check

web-build:
    cargo build -p whiskers-web-demo --lib --target wasm32-unknown-unknown --release
    wasm-bindgen target/wasm32-unknown-unknown/release/whiskers_web_demo.wasm --out-dir crates/whiskers-web-demo/web --out-name whiskers_web_demo --no-modules --no-typescript

web-build-opt: web-build
    wasm-opt -Os crates/whiskers-web-demo/web/whiskers_web_demo_bg.wasm -o crates7whiskers-web-demo/web/whiskers_web_demo_bg.wasm

web-serve:
    basic-http-server crates/whiskers-web-demo/web

test:
    cargo test

lint: clippy clippy-wasm fmt test
