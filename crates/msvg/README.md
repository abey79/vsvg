# *msvg*

This crate is part of the [*vsvg* project](https://github.com/abey79/vsvg).

**Status**: prototyping stage, aiming toward MVP.

## What's this?

Compared to *vpype*, *vsvg* is *extremely* fast to load and display SVGs. This makes a tool that can load a whole bunch of SVGs possible, for example to chose amongst many realisations of a generative art algorithm. This is what *msvg* aims to be.

## Installation

**WARNING**: this is at an early prototype stage!

To install `msvg`, you'll need Rust, which you can install using [`rustup`](https://www.rust-lang.org/learn/get-started).

Then, run the following command:

```
cargo install --git https://github.com/abey79/vsvg msvg
```


## Usage

```
msvg PATH [PATH...]
```

`PATH` may be a SVG file or a directory. If it is a directory, it will be recursively traversed and all founds will be included.
