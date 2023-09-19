# *vsvg*

## What's this?

This project started out as my Rust 🦀 learning journey and an exploration of how it could be used to power part of plotter-related projects [*vpype*](https://github.com/abey79/vpype) and/or [*vsketch*](https://github.com/abey79/vsketch). Though this is still accurate, *vsvg* has already turned into a number a promising—and sometimes already usable—parts.

- [**vsvg**](vsvg/README.md) is the core crate. It implements a Document/Layer/Path data structure suitable for plotter application. Compared to *vpype*, it introduces Béziers-based curved path element and per-path metadata, and generally is orders of magnitude faster. This crates include SVG I/O and algorithms such as path order optimisation, etc.
- [**vsvg-viewer**](vsvg-viewer/README.md) is a [egui](https://egui.rs)- and [wgpu](https://wgpu.rs)-based viewer for *vsvg*, very much like *vpype-viewer*. In addition to supporting the extra feature from *vsvg*, this viewer has the potential of being use on the web, as it can theoretically target WebAssembly. (This is still on the TODO list though.)
- [**vsvg-cli**](vsvg-cli/README.md) is an experimental, *vpype*-like CLI tool to manipulate SVGs from the terminal. Its purpose is mainly to serve as test bed for *vsvg* and *vsvg-viewer*. Eventually, this crate will disappear when *vpype*'s core will use *vsvg*.
- [**vsvg-multi**](vsvg-multi/README.md) will be a fast viewer/browser for SVG collection, based on *vsvg* and *vsvg-viewer*. It will smoothly address the challenge of browsing through large collections of generated SVGs, e.g. to find the best looking ones.
- [**vsvg-sketch**](vsvg-sketch/README.md) is a [Processing](https://processing.org)-like API to create generative plotter art, very much like *vsketch*, without the live-reload and parameters features (this is on the TODO list).

See the linked crates' README.md for more information.


<img width="500" alt="image" src="https://user-images.githubusercontent.com/49431240/220178589-e07f7e13-5706-4a7d-bbd4-aefffffa0c58.png">


## Is that the future of *vpype*?

Maybe. At least I'd very much like to.

*vpype* is currently made of two packages: `vpype` and `vpype-cli`. The former implements the "engine" and the API, that's then used by `vpype-cli` and plug-ins to offer a CLI interface. What this project explores is basically to opportunity to entirely re-implement the `vpype` package in Rust, which would aptly be renamed `vpype-core` in the process. This would dramatically improve the performance of *vpype*, thanks the Rust being compiled to native code and having much better concurrency.

The story for *vsketch* is a bit blurrier at this time. I imagine a complete rewrite using the new, fast `vpype-core`, a dynamic GUI using [`egui`](https://www.egui.rs), while keeping a Python front-end for the user (thanks to `vpype-core` Python bindings needed anyway to interface with `vpype-cli`). That would be lots of work, but the difference in performance would be dramatic, potentially enabling smooth animation and other cool stuff.

This would entail adding `vsk`-like APIs to `vpype-core` (see [#5](https://github.com/abey79/vsvg/issues/5)). Though this isn't needed for *vpype*, it would potentially enable writing *vpype* plugins in the style of *vsketch*, which sounds like a cool idea to me.

What's more like to happen in the short term is an improved version of *vsvg-sketch* to include *vsketch* style parameters in the UI. Iterating on code would still require a compilation cycle, but parameter exploration would be interactive.

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

### Running the sketch examples

See *vsvg-sketch*'s [README.md](vsvg-sketch/README.md).

### Installing *vsvg-cli*

See *vsvg-cli*'s [README.md](vsvg-cli/README.md).


## Design notes

A few design considerations can be found [here](https://github.com/abey79/vsvg/issues?q=is%3Aissue+is%3Aopen+label%3Adesign-note). They concern the use of this project as basis for a possible future Rust-based `vpype-core` package.

## Licence

This project is available under the MIT licence.
