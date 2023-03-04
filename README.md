# *vsvg*

## What's this?

This is a fast SVG viewer targeted at plotter users. It is somewhat usable, although it's still a highly experimental WIP. In addition, `vsvg` support a small subset of commands similar to *vpype*'s, albeit with a different syntax (see `vsvg --help`).

<img width="500" alt="image" src="https://user-images.githubusercontent.com/49431240/220178589-e07f7e13-5706-4a7d-bbd4-aefffffa0c58.png">

By "fast", I mean "much faster than [*vpype*](https://github.com/abey79/vpype)". Consider the following command:

```
vpype read 300_beziers.svg show
```

On my computer (a 2021 MacBook Pro M1 Max), *vpype* takes about 570ms *just to start*. Then it needs another 140ms to load this benchmark file containing 300 Bézier curves. Then it kicks off the viewer (this part is actually rather quick).

As of today `vsvg` takes ~3.6ms to start and load the same SVG file. It feels instantaneous.

In another test with a rather pathological 90MB SVG, `vsvg` takes 1.4s to load the file, where it takes upward of 30s for *vpype*. 

## Why?

I've recently started to learn Rust—and loving it so far! 🦀

This project serves me as a training ground and a mean of exploring if/how Rust could be used to power part of [*vpype*](https://github.com/abey79/vpype) and/or [*vsketch*](https://github.com/abey79/vsketch).

`vsvg` already supports features that would be highly beneficial to *vpype*, including:
- a data-model that includes Bézier curve (i.e. there is no loss of accuracy when loading SVG containing curved elements);
- a linearisation process (when one is needed, e.g. for display purposes) with an improved tolerance handling (the segment size adapts based on the curvature, which minimises the number of point needed when curves are nearly straight).
- some degree of per-path metadata handling (currently stroke color and width)

*vpype* is currently made of two packages: `vpype` and `vpype-cli`. The former implements the "engine" and the API, that's then used by `vpype-cli` and plug-ins to offer a CLI interface. What this project explores is basically to opportunity to entirely re-implement the `vpype` package in Rust, which would aptly be renamed `vpype-core` in the process. This would dramatically improve the performance of *vpype*, thanks the Rust being a compiled and making concurrency much easier.

The story for *vsketch* is a bit blurrier at this time. I imagine a complete rewrite using the new, fast `vpype-core`, a dynamic GUI using [`egui`](https://www.egui.rs), while keeping a Python front-end for the user (thanks to `vpype-core` Python bindings needed anyway to interface with `vpype-cli`). That would be lots of work, but the difference in performance would be dramatic, potentially enabling smooth animation and other cool stuff.

This would entail adding `vsk`-like APIs to `vpype-core` (see [#5](https://github.com/abey79/vsvg/issues/5)). Though this isn't needed for *vpype*, it would potentially enable writing *vpype* plugins in the style of *vsketch*, which sounds like a cool idea to me.

## Installing

There is currently no facilities to install `vsvg` unfortunately. It must be compiled and installed from source. Fortunately, this is actually not much more complicated than running a Python executable.

First, install Rust by running the command provided by [the official Rust website](https://www.rust-lang.org/tools/install):

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Normally, this will add `$HOME/.cargo/bin` in your path. 

Then, download the `vsvg` source code:

```
git clone https://github.com/abey79/vsvg
cd vsvg
```

Build and install `vsvg`:

```
cargo build --release
cargo install --path .
```

(If you cannot run `cargo`, your path hasn't been set correctly when installing Rust. Restart your terminal and try again. If it still fails, add `$HOME/.cargo/bin` in your PATH.)

`vsvg` should now be available:

```
vsvg --help
```

To uninstall `vsvg`, navigate back to the `vsvg` source directory and execute the following command:

```
cargo uninstall
```


## Design notes

A few design considerations can be found [here](https://github.com/abey79/vsvg/issues?q=is%3Aissue+is%3Aopen+label%3Adesign-note). They concern the use of this project as basis for a possible future Rust-based `vpype-core` package.

## TODO

- [x] Sort out page orientation and check that rotation, etc. work the same as with vpype
- [ ] ~~egui plot viewer cannot display zoom-aware fat lines :(~~ I'll deal with the viewer at a later stage—vpype 2 could keep the existing viewer. 
- [ ] ~~Properly handle Y axis (currently it's flipped)~~ (probably pointless if we move to a custom viewer)
  - [ ] ~~Custom y_axis_formatter~~
- [x] Add support for color and line width (but width is not zoom-aware)
- [x] Crop to page size
- [x] ~~Test viewbox~~ Fix viewbox handling
- [x] Fix missing top-level paths
- [ ] ~~Metadata concept, possibly using `Rc`'s clone-on-write capability~~ No! See [#4](https://github.com/abey79/vsvg/issues/4).
- [x] Split types.rs into multiple files (e.g. `types/document.rs`, `types/layer.rs`, etc.)
- [x] Move stuff to `lib.rs`
- [x] Implement *vpype*-like layer IDs.
- [ ] Rename `Path` to `Shape` to denote it being higher level?
- [x] Implement some Drawer API + add related commands?
- [x] Split into `core` and `gui` crates
- [ ] Make a "Quicklook" feature to browse SVGs??
- [ ] .......
