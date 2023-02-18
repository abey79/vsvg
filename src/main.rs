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

    /// return after loading the SVG file without showing the GUI (for benchmarking)
    #[clap(short, long)]
    no_gui: bool,

    /// tolerance when flatting SVG curves
    #[clap(short, long, default_value = "0.01")]
    tolerance: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let lines = parse_svg(cli.path, cli.tolerance)?;

    // Log to stdout (if you run with `RUST_LOG=debug`).
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::init();

    if !cli.no_gui {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "eframe template",
            native_options,
            Box::new(|cc| Box::new(viewer::Viewer::new(cc, lines))),
        )?;
    }
    Ok(())
}
