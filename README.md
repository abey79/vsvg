# The *vsvg* project (incl. *whiskers* and *msvg*)

## What's this?

|  [*whiskers*](crates/whiskers/README.md) | [*msvg*](crates/msvg/README.md) | [*vsvg*](crates/vsvg/README.md)/[*vsvg-viewer*](crates/vsvg-viewer/README.md) |
|---|---|---|
| <img width="300" alt="image" src="https://github.com/abey79/vsvg/assets/49431240/77adc4ba-a47d-4997-bcd5-3a56355bbd36"> |  <img width="300" alt="image" src="https://github.com/abey79/vsvg/assets/49431240/54c662f6-41c1-449f-954f-5d5c33a7c25b"> | <img width="300" alt="image" src="https://github.com/abey79/vsvg/assets/49431240/1c3d4096-9846-4902-a3f2-cc3ec43010a4"> |
| [*whiskers*](crates/whiskers/README.md) is a Rust-based, [Processing](https://processing.org)-like interactive sketching environment for generative plotter art. It's fast, it's web-ready, and it's a delight to use. <br/><br/>Try it [here](https://bylr.info/vsvg/)! | [*msvg*](crates/msvg/README.md) is a (WIP!) fast browser for SVG collections. It smoothly addresses the challenge of browsing through large collections of generated SVGs, e.g. to find the best looking ones for plotting. | [*vsvg*](crates/vsvg/README.md) and [*vsvg-viewer*](crates/vsvg-viewer/README.md) are the core crates behind *whiskers* and *msvg*. They implement the core data structures for manipulating vector data for plotter applications, as well as an ultra-performant, cross-platform, hardware-accelerated, and easy-to-extend viewer. |


## Documentation

The documentation is WIP—watch this space for updates.

In the meantime, each crate of the *vsvg* project has its own README with additional information:
- [*whiskers*](crates/whiskers/README.md)
- [*msvg*](crates/msvg/README.md)
- [*vsvg*](crates/vsvg/README.md)
- [*vsvg-viewer*](crates/vsvg-viewer/README.md)
- [*vsvg-cli*](crates/vsvg-cli/README.md)

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

See *whiskers*'s [README.md](crates/whiskers/README.md).

### Installing *vsvg-cli*

See *vsvg-cli*'s [README.md](crates/vsvg-cli/README.md).


## Design notes

A few design considerations can be found [here](https://github.com/abey79/vsvg/issues?q=is%3Aissue+is%3Aopen+label%3Adesign-note). They concern the use of this project as basis for a possible future Rust-based `vpype-core` package.

## Licence

This project is available under the MIT licence.
