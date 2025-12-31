install:
    cargo install --path crates/vsvg-cli

clippy $RUSTFLAGS="-Dwarnings":
    cargo clippy --workspace --all-features --bins --examples

clippy-wasm $RUSTFLAGS="-Dwarnings":
    cargo clippy --workspace --all-features --exclude msvg --exclude vsvg-cli --bins --target wasm32-unknown-unknown

docs $RUSTDOCFLAGS="-Dwarnings":
    cargo doc --all-features --no-deps --lib --bins --examples -p whiskers -p whiskers-widgets -p vsvg

fmt:
    cargo fmt --all -- --check

web-build:
    cargo build -p whiskers-web-demo --lib --target wasm32-unknown-unknown --release
    wasm-bindgen target/wasm32-unknown-unknown/release/whiskers_web_demo.wasm --out-dir crates/whiskers-web-demo/web --out-name whiskers_web_demo --no-modules --no-typescript

web-build-opt: web-build
    wasm-opt -Os crates/whiskers-web-demo/web/whiskers_web_demo_bg.wasm -o crates/whiskers-web-demo/web/whiskers_web_demo_bg.wasm

web-serve:
    basic-http-server crates/whiskers-web-demo/web

# Gallery build recipes
gallery-wasm:
    cargo build -p whiskers-gallery --lib --target wasm32-unknown-unknown --release
    wasm-bindgen target/wasm32-unknown-unknown/release/whiskers_gallery.wasm --out-dir crates/whiskers-gallery/web/sketches --out-name whiskers_gallery --no-modules --no-typescript

gallery-wasm-opt: gallery-wasm
    wasm-opt -Os crates/whiskers-gallery/web/sketches/whiskers_gallery_bg.wasm -o crates/whiskers-gallery/web/sketches/whiskers_gallery_bg.wasm

gallery-thumbnails:
    cargo run --bin generate-thumbnails --release

gallery-html:
    pip install -q -e crates/whiskers-gallery
    gallery-build

gallery-build: gallery-wasm-opt gallery-thumbnails gallery-html

gallery-serve: gallery-build
    python -m http.server -d crates/whiskers-gallery/web 8080

test:
    cargo test --workspace --all-features --bins --examples

doc-test:
    cargo test --doc --all-features

lint: clippy clippy-wasm fmt test doc-test docs
