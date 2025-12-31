# Whiskers Gallery

Interactive gallery of whiskers sketches, deployed at [whisk.rs](https://whisk.rs).

## Overview

This crate provides:

- A collection of example sketches demonstrating whiskers capabilities
- A web gallery with interactive WASM-based sketch viewers
- Source code viewing with syntax highlighting
- SVG thumbnail generation for the landing page

## Key Components

- **`sketches.toml`**: Single source of truth for sketch metadata (id, name, description, author)
- **`build.rs`**: Reads `sketches.toml` and generates Rust code (WASM entry points, `render_sketch()` function)
- **`src/sketches/`**: Sketch implementations, each exposing a `runner()` function
- **`gallery_builder/`**: Python package with Jinja2 templates for HTML generation
- **`generate-thumbnails` binary**: Renders each sketch headlessly to SVG for the landing page

## Building

```bash
# Full build (WASM + thumbnails + HTML)
just gallery-build

# Individual steps
just gallery-wasm        # Build WASM binary
just gallery-thumbnails  # Generate SVG thumbnails
just gallery-html        # Generate HTML pages

# Local development server
just gallery-serve       # Builds and serves at http://localhost:8080
```

## Maintainer's Guide

### Adding a New Sketch

1. **Create the sketch module** at `src/sketches/{sketch_id}.rs`.
2. **Add the module** in `src/sketches/mod.rs`.
3. **Add metadata** to `sketches.toml`.
4. **Rebuild**:

   ```bash
   just gallery-build
   ```

   This will:
   - Regenerate `src/generated.rs` with WASM entry points and the render function
   - Generate the SVG thumbnail
   - Generate the HTML pages

### Updating a Sketch

- **Code changes**: Just rebuild with `just gallery-build`
- **Metadata changes**: Edit `sketches.toml`, then rebuild
- **Thumbnail refresh**: Run `just gallery-thumbnails`

### How Code Generation Works

`build.rs` reads `sketches.toml` and generates `src/generated.rs` containing:

- `SKETCH_MANIFEST`: Static array of sketch metadata
- `render_sketch(id, seed)`: Native function to render any sketch headlessly (used for thumbnails)
- `start_{id}()`: WASM entry points for each sketch

This ensures the manifest is the single source of truth - no need to update multiple files when adding sketches.

### CI Checks

The CI verifies that `src/generated.rs` is up to date:

```bash
cargo build -p whiskers-gallery
git diff --exit-code crates/whiskers-gallery/src/generated.rs
```

If this fails, run `just gallery-build` locally and commit the updated `generated.rs`.
