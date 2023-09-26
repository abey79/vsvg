# *vsvg-viewer* crate

This crate is part of the [*vsvg* project](https://github.com/abey79/vsvg).

**Status**: Beta. It works, but the API is subject to change.

## What's this?

This crate implements an extensible [egui](https://egui.rs)- and [wgpu](https://wgpu.rs)-based viewer for *vsvg*. It's very much similar to *vpype-viewer*, but built on much stronger foundations:

- The egui crate is an [immediate-mode](https://en.wikipedia.org/wiki/Immediate_mode_GUI) UI framework, which makes it suited for easy customisation and highly interactive projects (such as [*whiskers*](../whiskers/README.md)).
- The wgpu crate is a wrapper over modern native *and web* graphics APIs. It's future-proof.

This combination enables targeting WebAssembly too.

## Usage

*vsvg-viewer* offers two main features:
- a basic `vsvg::Document` viewer
- a fully customisable application built on the `vsvg::Document` viewer

### Basic viewer

The basic `vsvg::Document` viewer requires a single line of code:

```rust
fn main() -> anyhow::Result<()> {
    let doc = vsvg::Document::from_svg("path/to/input.svg");
    vsvg_viewer::show(&doc)
}
```

Here is an example screenshot:

<img width="912" alt="image" src="https://github.com/abey79/vsvg/assets/49431240/3659fa9a-a967-41d7-bede-743e025e748e">

For users familiar with *vpype*, this basically corresponds to what happens with `vpype [...] show`.


### Customisable app

Alternatively, *vsvg-viewer* can be used to build complex, interactive application around the core `vsvg::Document` renderer feature:

```rust
struct MyApp {}

impl vsvg_viewer::ViewerApp for MyApp {
    /* ... */
}

fn main() -> anyhow::Result<()> {
    vsvg_viewer::show_with_viewer_app(MyApp::new())
}
```

Your app must implement the `vsvg_viewer::ViewerApp` trait, which offers hooks to:
- display the custom UI, for example in side panels;
- customise options such as windows title, etc.;
- load/save state from persistent storage.

A basic example for a custom app is provided in the `examples/` directory.

[*whiskers*](../whiskers/README.md)) uses this API to implement its sketch runner:

<img width="1237" alt="image" src="https://github.com/abey79/vsvg/assets/49431240/636a343b-d175-4b8a-9acf-812ae64f2b32">
