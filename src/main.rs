mod crop;
mod svg_reader;
mod types;
mod viewer;

use crate::svg_reader::*;
use std::error::Error;
use std::hint::black_box;

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
    // Log to stdout (if you run with `RUST_LOG=debug`).
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::init();

    // load SVG file
    let cli = Cli::parse();
    let doc = parse_svg(cli.path)?;

    // flatten curves
    let lines = doc.flatten(cli.tolerance);

    if !cli.no_gui {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "eframe template",
            native_options,
            Box::new(|cc| Box::new(viewer::Viewer::new(cc, lines))),
        )?;
    } else {
        // ensure this is not optimised away
        // TODO: needs to be eventually cleaned up
        black_box(lines);
    }
    Ok(())
}
