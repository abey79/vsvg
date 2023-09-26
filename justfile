install:
    cargo install --path vsvg

clippy:
    cargo clippy --workspace --bins --examples

clippy-wasm:
    cargo clippy --workspace --exclude vsvg-multi --exclude vsvg-cli --bins --target wasm32-unknown-unknown


web-build:
    cargo build -p whiskers-web-demo --lib --target wasm32-unknown-unknown --release
    wasm-bindgen target/wasm32-unknown-unknown/release/whiskers_web_demo.wasm --out-dir whiskers-web-demo/web --out-name whiskers_web_demo --no-modules --no-typescript
    wasm-opt -O3 whiskers-web-demo/web/whiskers_web_demo_bg.wasm -o whiskers-web-demo/web/whiskers_web_demo_bg_opt.wasm

web-serve:
    basic-http-server whiskers-web-demo/web
