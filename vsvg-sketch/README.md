# *vsvg-sketch* crate

This crate is part of the [*vsvg* project](https://github.com/abey79/vsvg).

**Status**: WIP and undocumented, but already usable by learning from the examples.


## What's this?

*vsvg-sketch* is a [Processing](https://processing.org)-like sketch API built over *vsvg*, similar to *vsketch*'s `vsketch.Vsketch` API.

Here is how it looks with a simple example:

```rust
use vsvg_sketch::prelude::*;

fn main() -> Result {
    Sketch::with_page_size(PageSize::A5)
        .scale(Units::CM)
        .translate(7.0, 6.0)
        .circle(0.0, 0.0, 2.5)
        .translate(1.0, 4.0)
        .rotate_deg(45.0)
        .rect(0., 0., 4.0, 1.0)
        .show()?
        .save("output.svg")?;

    Ok(())
}
```

It's still very basic, but surprisingly usable. I've already integrated into my [Rusteroïds](https://github.com/abey79/rusteroids) toy game for the generative asteroids.


## Roadmap

Contrary to *vsketch*, *vsvg-sketch* doesn't offer any IDE-like features such as live reload and UI parameters.

As a first step, I would like to introduce parameters to the API and add the corresponding interactive UI. Given how much faster *vsvg* generally is, this should make for a very nice experience. I'll also introduce a "time" default parameter, which can be used to animate things, along with play/pause/back controls to find the perfect frame to plot. Given *vsvg-viewer*'s foundation–in particular the [egui framework](https://egui.rs) that I now use professionally—this should be vastly easier to build than *vsketch* was.

As a later step, I would like to bring back a Python-based coating, for ease of use and the bring back the live reload ability. I'm not sure yet how/if the details of that will pan out.

Another avenue that I want to explore is targeting WebAssembly for web deployment, which is in theory relatively easy to do with *vsvg*'s stack. Once UI parameters are supported, it means that any *vsvg-sketch*-based sketch could be turned into a SVG generator web app. Combined with GitHub Pages, that would be done in a Single Click (tm) and with free hosting 🤩



## Running examples

To run the example, use the following command (in this case to run `vsvg-sketch/examples/basic.rs`):

```
cargo run --package vsvg-sketch --example basic
```
