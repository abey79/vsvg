# *vsvg-viewer* crate

This crate is part of the [*vsvg* project](https://github.com/abey79/vsvg).

**Status**: Experimental, but works really neatly already, and a sound foundation for the future.

## What's this?

This crate implements a [egui](https://egui.rs)- and [wgpu](https://wgpu.rs)-based viewer for *vsvg*. It's very much similar to *vpype-viewer*, but built on much stronger foundations:

- The egui crate is an [immediate-mode](https://en.wikipedia.org/wiki/Immediate_mode_GUI) UI framework, which makes it highly suited for interactive projects (think of *vsketch*'s interactive parameters).
- The wgpu crate is a wrapper over native *and web* most modern graphics API. It's future-proof.

This combination enables targeting WebAssembly too.
