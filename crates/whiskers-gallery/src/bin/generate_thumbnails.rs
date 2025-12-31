//! Generate SVG thumbnails for all gallery sketches.
//!
//! This binary renders each sketch headlessly and exports SVG thumbnails
//! to the web/thumbnails directory. It reads the sketch manifest to determine
//! which sketches to render.

use anyhow::Result;
use std::path::PathBuf;
use vsvg::DocumentTrait;

use whiskers_gallery::{SKETCH_MANIFEST, render_sketch};

fn main() -> Result<()> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let thumbnails_dir = crate_root.join("web").join("thumbnails");
    std::fs::create_dir_all(&thumbnails_dir)?;

    println!("Generating thumbnails in {:?}", thumbnails_dir);

    for sketch in SKETCH_MANIFEST {
        let mut document = render_sketch(sketch.id, 0)
            .unwrap_or_else(|| panic!("Unknown sketch: {}", sketch.id))?;
        // Disable date in SVG metadata for deterministic output
        document.metadata_mut().include_date = false;
        let svg = document.to_svg_string()?;
        let path = thumbnails_dir.join(format!("{}.svg", sketch.id));
        std::fs::write(&path, &svg)?;
        println!("  Generated: {}", path.display());
    }

    println!("Done!");
    Ok(())
}
