# *vsvg-cli* crate

This crate is part of the [*vsvg* project](https://github.com/abey79/vsvg).

**Status**: Highly experimental, and likely superseeded in the future by a *vsvg*-powered *vpype* 2.0. Still somewhat usable.

## What's this?

*vsvg-cli* is a *vpype*-like pipeline-ish CLI wrapper over *vsvg*. Its primary purpose is to serve as test bed for *vsvg* and *vsvg-viewer*.


## Installation

### From pre-built binaries

A number of pre-built binaries and installers are available on the [Release](https://github.com/abey79/vsvg/releases/latest) page, including shell/PowerShell-based installers, binary archives for most platforms, and MSI archives for Windows.


### From source

To install the `vsvg` CLI, you'll need Rust, which you can install using [`rustup`](https://www.rust-lang.org/learn/get-started).

Then, run the following command:

```
cargo install vsvg-cli
```

Build and install *vsvg-cli*:


The `vsvg` command should now be available:

```
vsvg --help
```

To uninstall the `vsvg` CLI:

```
cargo uninstall vsvg-cli
```

## Documentation

No documentation is currently available for *vsvg-cli*, and likely none will ever be. Use the integrated help:

```
vsvg --help
```

This is the output as of mid-September 2023:

```
An experimental SVG viewer for plotter users.

Usage: vsvg [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the SVG file (or '-' for stdin)

Options:
      --single   Single layer mode
      --no-show  Don't show the GUI
  -v, --verbose  Enable debug output
  -h, --help     Print help
  -V, --version  Print version

COMMANDS:
  -t, --translate <X> <X>
          Translate by provided coordinates
  -R, --rotate-rad <X>
          Rotate by X radians around the origin
  -r, --rotate <X>
          Rotate by X degrees around the origin
  -s, --scale <X>...
          Uniform (X) or non-uniform (X Y) scaling around the origin
      --scale-around <X> <X> <X> <X>
          Scale around the provided point
  -c, --crop <X> <X> <X> <X>
          Crop to provided XMIN, YMIN, XMAX, YMAX
      --linesort <FLIP>
          Reorder paths to minimize pen-up distance [possible values: true, false]
      --linesortnoflip <THRES>
          Reorder paths to minimize pen-up distance
      --linesortflip <THRES>
          Reorder paths to minimize pen-up distance
      --dlayer <X>
          Set target layer for draw operations
      --dtranslate <X> <X>
          Apply an X, Y translation to the current transform
      --drotate <X>
          Apply a rotation to the current transform
      --dscale <X>...
          Apply a uniform (X) or non-uniform (X, Y) scale to the current transform
      --dskew <X> <X>
          Apply a (X, Y) skew to the current transform
      --dcbez <X> <X> <X> <X> <X> <X> <X> <X>
          Draw a cubic bezier curve with X, Y, X1, Y1, X2, Y2, X3, Y3
      --dqbez <X> <X> <X> <X> <X> <X>
          Draw a quadratic bezier curve with X, Y, X1, Y1, X2, Y2
      --darc <X> <X> <X> <X> <X> <X> <X>
          Draw an arc with X, Y, RX, XY, START, SWEEP, ROT_X
      --dcircle <dcircle> <dcircle> <dcircle>
          Draw a circle with X, Y, R
      --dellipse <dellipse> <dellipse> <dellipse> <dellipse> <dellipse>
          Draw an ellipse with X, Y, RX, RY, ROT_X
      --dline <X> <X> <X> <X>
          Draw a line with X, Y, X1, Y1
      --drect <X> <X> <X> <X>
          Draw a rectangle with X, Y, W, H
      --drrect <X> <X> <X> <X> <X> <X> <X> <X>
          Draw a rounded rectangle with X, Y, W, H, TL, TR, BR, BL
      --dsvg <X>
          Draw from an SVG path representation
      --write <FILE>
          Write the current document to a file
      --stats <stats>
          Print stats [possible values: true, false]

```
