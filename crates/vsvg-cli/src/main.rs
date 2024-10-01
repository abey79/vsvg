mod cli;
mod commands;
mod draw_state;

use std::error::Error;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> Result<(), Box<dyn Error>> {
    cli::cli()?;

    Ok(())
}
