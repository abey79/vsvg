# *whiskers*: interactive sketches for plotter generative art

[<img alt="github" src="https://img.shields.io/badge/github-abey79/vsvg-8da0cb?logo=github" height="20">](https://github.com/abey79/vsvg)
[![Latest version](https://img.shields.io/crates/v/whiskers.svg)](https://crates.io/crates/whiskers)
[![Documentation](https://docs.rs/whiskers/badge.svg)](https://docs.rs/whiskers)
[![GitHub](https://img.shields.io/github/license/abey79/vsvg)](https://github.com/abey79/vsvg/blob/master/LICENSE)


_👉 Try the [**live demo**](https://bylr.info/vsvg/)!_


*whiskers* is a [Processing](https://processing.org)-like interactive sketch environment and API built over [*vsvg*](../vsvg/README.md) and [*vsvg-viewer*](../vsvg-viewer/README.md). It's similar to *vsketch*, but faster, web-ready, and with much stronger foundations.

<img width="1116" alt="image" src="https://github.com/abey79/vsvg/assets/49431240/bfecf0a1-a0a1-4d27-8a42-6ad95ac438fa">

## Installation

```
cargo add whiskers
```

## Usage

Here is the code for this basic sketch:

```rust
use whiskers::prelude::*;

#[derive(Sketch)]  // <- this is the magic to make the sketch interactive
struct HelloWorldSketch {
    width: f64,   // <- interactive UI is automagically built for these fields
    height: f64,
}

impl Default for HelloWorldSketch {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 300.0,
        }
    }
}

impl App for HelloWorldSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        sketch.color(Color::DARK_RED).stroke_width(3.0);

        // the `Sketch` API is a delight to work with
        sketch
            .translate(sketch.width() / 2.0, sketch.height() / 2.0)
            .rect(0., 0., self.width, self.height);

        Ok(())
    }
}

fn main() -> Result {
    Runner::new(HelloWorldSketch::default())
        .with_page_size_options(PageSize::A5H)
        .run()
}
```

*whiskers* is part of the [*vsvg* project](https://github.com/abey79/vsvg).

## Features

- [x] Interactive UI automatically built based on the sketch `struct`'s fields.
- [x] Sketch parameter UI highly customisable using `#[param(...)]` attributes (see e.g. `asteroid` example).
- [x] Sketch parameter UI easily extendable for custom data types (see e.g. `custom_ui` example).
- [x] Page size management UI.
- [x] Export to SVG
- [x] Time parameter management UI (for animated sketches).
- [x] Random Number Generator UI with seed control (see e.g. `asteroid` example).
- [ ] Configuration handling (save/restore config, etc.)
- [ ] Compiled sketches are *also* a flexible CLI utility with the capability to batch generate sketch outputs with parameter ranges.
- [ ] Export to other format through templating (HPGL, g-code, etc. — for now, please use [*vpype*](https://github.com/abey79/vpype)).
- [ ] ... (*please complete this list*)


## Isn't that *vsketch*?

Compared to [*vsketch*](https://github.com/abey79/vsketch), *whiskers* offers the following advantages:

- It's in Rust, so it's faaast! 🚀
- Sketches can be compiled to WebAssembly and published on the Web (try it [here](https://bylr.info/vsvg/)).
- It's built on a stack (mainly [egui](https://egui.rs) and [wgpu](https://wgpu.rs)) that's *much* faster and easier to work with.

On the other hand:

- It's in Rust, which has a steeper learning curve than Python.
- Since sketches are compiled, the edit code/execute cycle is heavier than with *vsketch*.

I have the vague dream to slap a Python front-end on top of *whiskers* to build *vsketch* 2.0, but that's a lot of work and other things have a higher priority, so no promises on this front.


## Running examples

To run the example, use the following command (in this case to run `crates/whiskers/examples/asteroid.rs`):

```
cargo run --release --package whiskers --example asteroid
```

## Non-interactive use

The `whiskers::Sketch` object can be used stand-alone, without the interactive sketch runner. This is useful if you want to use the drawing capabilities in your own tools.

For example, I use `whiskers::Sketch` to build the generative asteroids in my [Rusteroïds](https://github.com/abey79/rusteroids) toy game.

Here is how the code could look:

```rust
use whiskers::prelude::*;

fn main() -> Result {
    Sketch::with_page_size_options(PageSize::A5)
        .scale(Units::Cm)
        .translate(7.0, 6.0)
        .circle(0.0, 0.0, 2.5)
        .translate(1.0, 4.0)
        .rotate_deg(45.0)
        .rect(0., 0., 4.0, 1.0)
        .save("output.svg")?;

    Ok(())
}
```

If the `viewer` feature of *whiskers is enabled (which it is by default), the sketch can be displayed using the basic viewer using `sketch.show()`.
