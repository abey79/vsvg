# *msvg*

This crate is part of the [*vsvg* project](https://github.com/abey79/vsvg).

**Status**: functional but very beta.

## What's this?

Compared to *vpype*, *vsvg* is *extremely* fast to load and display SVGs. This makes a tool that can load a _whole bunch_ of SVGs possible, for example to chose amongst many realisations of a generative art algorithm. This is what *msvg* aims to be.

https://github.com/abey79/vsvg/assets/49431240/817f7dbe-2562-4c4d-9ab2-41368ed60677

## Installation


### From pre-built binaries

A number of pre-built binaries and installers are available on the [Release](https://github.com/abey79/vsvg/releases/latest) page, including shell/PowerShell-based installers, binary archives for most platforms, and MSI archives for Windows.


### From source

To install `msvg`, you'll need Rust, which you can install using [`rustup`](https://www.rust-lang.org/learn/get-started).

Then, run the following command:

```
cargo install --git https://github.com/abey79/vsvg msvg
```


## Usage

```
msvg PATH [PATH...]
```

`PATH` may be an SVG file or a directory. If it is a directory, it will be recursively traversed and all founds will be included.
