mod svg_reader;
mod viewer;

use crate::svg_reader::*;
use std::error::Error;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// path of input SVG file
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    println!("{:?}", cli.path);

    let lines = parse_svg(cli.path)?;

    // Log to stdout (if you run with `RUST_LOG=debug`).
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(viewer::Viewer::new(cc, lines))),
    )?;

    Ok(())
}
