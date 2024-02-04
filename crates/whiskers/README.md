# *whiskers*: interactive sketches for plotter generative art

[<img alt="github" src="https://img.shields.io/badge/github-abey79/vsvg-8da0cb?logo=github" height="20">](https://github.com/abey79/vsvg)
[![Latest version](https://img.shields.io/crates/v/whiskers.svg)](https://crates.io/crates/whiskers)
[![Documentation](https://docs.rs/whiskers/badge.svg)](https://docs.rs/whiskers)
[![GitHub](https://img.shields.io/github/license/abey79/vsvg)](https://github.com/abey79/vsvg/blob/master/LICENSE)


*whiskers* is a Rust interactive sketch environment for plotter generative art, with a [Processing](https://processing.org) inspired API.

_👉 Try the [**live demo**](http://whisk.rs/)!_

It's similar to [vsketch](https://github.com/abey79/vsketch), but faster, web-ready, and with [*vsvg*](../vsvg/README.md) as a much stronger foundation.

<img width="1062" alt="image" src="https://github.com/abey79/vsvg/assets/49431240/57ea7a5e-1c46-4a86-8155-2b0a217e6817">

## Installation

Simply add *whiskers* as a dependency to your project:

```
cargo add whiskers
```

## Usage

Here is the code for a basic sketch:

```rust
use whiskers::prelude::*;

#[sketch_app]  // <- this is the magic to make the sketch interactive
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
        // this is where the drawing happens:
        sketch
            .translate(sketch.width() / 2.0, sketch.height() / 2.0)
            .rect(0., 0., self.width, self.height);

        Ok(())
    }
}

fn main() -> Result {
    HelloWorldSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .run()
}
```

This is the result:

<img width="985" alt="image" src="https://github.com/abey79/vsvg/assets/49431240/2a7f8cac-0206-44c6-b471-296f1487fc26">

## Features

*whiskers* is a work in progress, but currently features:

- [x] Interactive and highly customisable interactive UI.
- [x] Export to SVG.
- [x] Drawing with lines, shapes, svg paths.
- [x] Support for curves: quadratic Béziers, cubic Bézier, Catmull-Rom splines—circles. (Ellipses and arcs are supported but internally converted to cubic Bézier.)
- [x] Transformations such as translations, rotations, scaling.
- [x] Pen settings such as line width, color, opacity, layers.
- [x] Grid helpers for rectangular or hexagonal grid based sketches.
- [x] Animated sketches support.
- [x] Random Number Generator UI with seed control (see e.g. `asteroid` example).
- [x] Integrated profiler (based on [puffin](https://github.com/EmbarkStudios/puffin)).
- [x] Web assembly compatibility, export your sketch UI for browsers ([demo](http://whisk.rs/))
- [ ] Configuration handling (save/restore config, etc.).
- [ ] Compiled sketches are *also* a flexible CLI utility with the capability to batch generate sketch outputs with parameter ranges.
- [ ] Export to other format through templating (HPGL, g-code, etc. — for now, please use [*vpype*](https://github.com/abey79/vpype)).
- [ ] ... (*please complete this list*)

On top of all that, you can import other rust packages for features such as noise and boolean operations, for which you can use `noise-rs` and `geo` respectively (see examples `noise` and `astroids`)..

## Isn't that *vsketch*?

Compared to [*vsketch*](https://github.com/abey79/vsketch), *whiskers* offers the following advantages:

- It's in Rust, so it's faaast! 🚀
- Sketches can be compiled to WebAssembly and published on the Web (try it [here](http://whisk.rs/)).
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

## Call to action

This project is at an early the stage and needs your contribution. Please get in touch via [discussions on GitHub](https://github.com/abey79/vsvg/discussions) or the [DrawingBots’s Discord server](https://discord.com/invite/XHP3dBg) if you feel like helping!
