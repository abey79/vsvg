//! Build script for whiskers-gallery.
//!
//! Reads `sketches.toml` and generates `src/generated.rs` containing:
//! - SKETCH_MANIFEST: metadata for all sketches
//! - WASM entry points: start_* functions for each sketch

use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct Sketch {
    id: String,
    name: String,
    description: String,
    author: String,
}

fn parse_sketches(toml_content: &str) -> Vec<Sketch> {
    let mut sketches = Vec::new();
    let mut current: Option<Sketch> = None;

    for line in toml_content.lines() {
        let line = line.trim();

        if line == "[[sketch]]" {
            if let Some(sketch) = current.take() {
                sketches.push(sketch);
            }
            current = Some(Sketch {
                id: String::new(),
                name: String::new(),
                description: String::new(),
                author: String::new(),
            });
        } else if let Some(ref mut sketch) = current {
            if let Some(value) = line.strip_prefix("id = ") {
                sketch.id = value.trim_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("name = ") {
                sketch.name = value.trim_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("description = ") {
                sketch.description = value.trim_matches('"').to_string();
            } else if let Some(value) = line.strip_prefix("author = ") {
                sketch.author = value.trim_matches('"').to_string();
            }
        }
    }

    if let Some(sketch) = current {
        sketches.push(sketch);
    }

    sketches
}

fn generate_code(sketches: &[Sketch]) -> String {
    let mut code = String::new();

    // Header
    code.push_str(
        "// AUTO-GENERATED FROM sketches.toml - DO NOT EDIT
// Run `cargo build -p whiskers-gallery` to regenerate

use crate::sketches::SketchMeta;

/// Registry of all available sketches.
pub static SKETCH_MANIFEST: &[SketchMeta] = &[
",
    );

    // Manifest entries
    for sketch in sketches {
        code.push_str(&format!(
            r#"    SketchMeta {{
        id: "{}",
        name: "{}",
        description: "{}",
        author: "{}",
    }},
"#,
            sketch.id, sketch.name, sketch.description, sketch.author
        ));
    }

    code.push_str("];\n");

    // Native: render a sketch headlessly by ID (for thumbnail generation, etc.)
    code.push_str(
        "
/// Render a sketch headlessly by ID and return the resulting document.
#[cfg(not(target_arch = \"wasm32\"))]
pub fn render_sketch(id: &str, seed: u32) -> Option<anyhow::Result<vsvg::Document>> {
    match id {
",
    );

    for sketch in sketches {
        code.push_str(&format!(
            r#"        "{id}" => Some(crate::sketches::{id}::runner().run_headless(seed)),
"#,
            id = sketch.id
        ));
    }

    code.push_str(
        "        _ => None,
    }
}
",
    );

    // WASM entry points
    code.push_str(
        "
#[cfg(target_arch = \"wasm32\")]
mod wasm {
    use wasm_bindgen::prelude::*;
",
    );

    for sketch in sketches {
        code.push_str(&format!(
            r#"
    #[wasm_bindgen]
    pub async fn start_{id}(
        handle: &vsvg_viewer::web_handle::WebHandle,
        canvas: vsvg_viewer::exports::web_sys::HtmlCanvasElement,
    ) -> Result<(), JsValue> {{
        handle.start(canvas, crate::sketches::{id}::runner()).await
    }}
"#,
            id = sketch.id
        ));
    }

    code.push_str("}\n");

    code
}

fn main() {
    println!("cargo:rerun-if-changed=sketches.toml");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml_path = Path::new(&manifest_dir).join("sketches.toml");
    let output_path = Path::new(&manifest_dir).join("src/generated.rs");

    let toml_content = fs::read_to_string(&toml_path).expect("Failed to read sketches.toml");

    let sketches = parse_sketches(&toml_content);

    let code = generate_code(&sketches);

    // Only write if content changed (avoids unnecessary rebuilds)
    let should_write = match fs::read_to_string(&output_path) {
        Ok(existing) => existing != code,
        Err(_) => true,
    };

    if should_write {
        fs::write(&output_path, &code).expect("Failed to write generated.rs");
        println!(
            "cargo:warning=Generated {} with {} sketches",
            output_path.display(),
            sketches.len()
        );
    }
}
