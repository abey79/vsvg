# 0.4.0 - The Too Late for Genuary Release

Released 2024-01-21

## Highlights

### Parameter persistence and sketch breaking changes

Sketch parameters are now persisted across sketch launches, which dramatically improves the QoL when iterating on sketch code. A new "Reset" button sets all parameters to the sketch defaults. This also lays the groundwork for a future configuration management feature.

Two breaking changes were introduced to accommodate for this.

1) Use `MySketch::runner().run()` instead of `Runner::new(MySketch::default()).run()` in your `main()` function.
2) Use the `#[sketch_app]`, resp. `#[sketch_widget]` attributes instead of the `Sketch`, resp. `Widget` derive macros. These new attributes take care of deriving the now-required `serde::Serialize` and `serde::Deserialize` using the correct, `whiskers`-exported version of the `serde` crate.

Example:
```rust
use whiskers::prelude::*;

#[sketch_app]
#[derive(Default)]
struct HelloWorldSketch {
    width: f64,
    height: f64,
}

impl App for HelloWorldSketch {
    fn update(&mut self, sketch: &mut Sketch, _ctx: &mut Context) -> anyhow::Result<()> {
        /* sketch code here */
        Ok(())
    }
}

fn main() -> Result {
    HelloWorldSketch::runner()
        .with_page_size_options(PageSize::A5H)
        .run()
}
```

### Other highlights

- It's now possible to override pen width and opacity in the viewer.
- The viewer now persists the antialiasing setting (note: persistence happens on a per-binary basis, so the AA setting must be set e.g. for each different sketch).
- You can use custom `enum` types as sketch parameter (use the new `#[sketch_widget]` attribute as noted above).
- `vsvg` introduces a new `Length` type which combines a float and a `Unit`. `whiskers` supports them, and, when used as sketch parameter, provides a nice UI where both the value and the unit can be changed.
- `msvg` now sorts files "correctly" when they are numbered, and has a much nicer CLI experience.
- It's now possible to directly "draw" into a `vsvg::Layer` using the APIs from the `vsvg::Draw` trait.
- Both `vsvg` and `vsvg-viewer` now cleanly re-export key dependencies.


## `whiskers` crates

- **BREAKING:** Persist sketch parameters across app relaunches [#94](https://github.com/abey79/vsvg/pull/94)
- **BREAKING:** Add a button to reset the sketch parameters to their defaults [#91](https://github.com/abey79/vsvg/pull/91)
- Add support for custom `enum` as sketch parameter [#107](https://github.com/abey79/vsvg/pull/107)
- Add support for `vsvg::Unit` and `vsvg::Length` as sketch parameters [#95](https://github.com/abey79/vsvg/pull/95)
- Add `Context::rng_weighted_choice()` helper function [#102](https://github.com/abey79/vsvg/pull/102) (thanks [@afternoon2](https://github.com/afternoon2)!)
- Split whiskers widgets in their own `whiskers-widgets` crate [#108](https://github.com/abey79/vsvg/pull/108)
- Add `particle` example based on `geos` [#105](https://github.com/abey79/vsvg/pull/105)
- Make `Runner::new()` private and update docs accordingly [#96](https://github.com/abey79/vsvg/pull/96)
- Fix `README.md` code example to use `SketchApp::runner()` instead of now private `Runner::new()` [#103](https://github.com/abey79/vsvg/pull/103) (thanks [@reidab](https://github.com/reidab)!)

## `msvg` CLI

- Sort files in natural order rather than in lexicographical order [#104](https://github.com/abey79/vsvg/pull/104) (thanks [@danieledapo](https://github.com/danieledapo)!)
- Use `clap` for `msvg` for a nicer CLI experience [#83](https://github.com/abey79/vsvg/pull/83)

## `vsvg` crate

- **BREAKING:** Improve `Unit` and introduce `Length` [#88](https://github.com/abey79/vsvg/pull/88)
- Implement the `Draw` trait for `Layer` [#111](https://github.com/abey79/vsvg/pull/111)
- Re-export core `vsvg` dependencies [#113](https://github.com/abey79/vsvg/pull/113)
- Fix unescaped `<dc:source>` content in SVG output [#116](https://github.com/abey79/vsvg/pull/116)

## `vsvg-viewer` crate

- Add options to override pen width and opacity [#89](https://github.com/abey79/vsvg/pull/89)
- Persist antialias setting across app relaunches [#90](https://github.com/abey79/vsvg/pull/90)
- Add `on_exit()` hook to the `ViewerApp` trait [#106](https://github.com/abey79/vsvg/pull/106) (thanks [@danieledapo](https://github.com/danieledapo)!)
- Re-export core `vsvg-viewer` dependencies [#115](https://github.com/abey79/vsvg/pull/115) (thanks [@danieledapo](https://github.com/danieledapo)!)

## Common

- Run documentation tests in CI [#92](https://github.com/abey79/vsvg/pull/92)
- Update rust toolchain to 1.75.0 [#82](https://github.com/abey79/vsvg/pull/82)
- Update egui to 0.25.0 [#118](https://github.com/abey79/vsvg/pull/118)
- Update `cargo dist` to 0.8.0 [#117](https://github.com/abey79/vsvg/pull/117)
- Fix web demo publishing action [`5f42b4a`](https://github.com/abey79/vsvg/commit/5f42b4aaf4bece6aa914fe3f1037a11a139feb98)
- `changelog.py`: highlight breaking changes and generate a list of contributors [#93](https://github.com/abey79/vsvg/pull/93)

## Contributors

[<img src="https://wsrv.nl/?url=github.com/afternoon2.png?w=64&h=64&mask=circle&fit=cover&maxage=1w" width="32" height="32" alt="afternoon2" />](https://github.com/afternoon2) [<img src="https://wsrv.nl/?url=github.com/danieledapo.png?w=64&h=64&mask=circle&fit=cover&maxage=1w" width="32" height="32" alt="danieledapo" />](https://github.com/danieledapo) [<img src="https://wsrv.nl/?url=github.com/reidab.png?w=64&h=64&mask=circle&fit=cover&maxage=1w" width="32" height="32" alt="reidab" />](https://github.com/reidab)

**Full Changelog**: https://github.com/abey79/vsvg/compare/v0.3.0...v0.4.0


# 0.3.0 - New `msvg` CLI, better `whiskers`, and more

Released 2023-12-28

## Highlights

- Inspect SVG collections with the new, blazing fast `msvg` CLI (early alpha stage).
- `whiskers` improvements:
  - New hexagonal grid helper.
  - Support for nested `struct` in sketch param.

## `whiskers` crates

- Add support for custom `struct` as sketch parameter [#66](https://github.com/abey79/vsvg/pull/66)
- Add hexagonal grid helper [#60](https://github.com/abey79/vsvg/pull/60) (thanks [@afternoon2](https://github.com/afternoon2)!)
- Change `HexGrid::spacing()` to accept a single scalar and maintain hexagonal grid [#72](https://github.com/abey79/vsvg/pull/72) (thanks [@karliss](https://github.com/karliss)!)
- Implement `step` UI parameter for numeric value in normal mode [#58](https://github.com/abey79/vsvg/pull/58)

## `msvg` CLI

- First prototype of `msvg` [#68](https://github.com/abey79/vsvg/pull/68)
- Improve `msvg`'s file list side panel and add file name overlay [#76](https://github.com/abey79/vsvg/pull/76)
- Fix blank window on first start [#81](https://github.com/abey79/vsvg/pull/81)

## `vsvg` CLI

- Add "merge layers" operation to `vsvg` and `vsvg-cli` [#61](https://github.com/abey79/vsvg/pull/61)
- Add `--strokewidth <W>` command to override the stroke width of all paths [#62](https://github.com/abey79/vsvg/pull/62)
- Add `--flatten <TOL>` command to flatten all curves with the provided tolerance [#63](https://github.com/abey79/vsvg/pull/63)

## `vsvg-viewer` crate

- Improve `ViewerApp` hooks to give implementers more flexibility [#71](https://github.com/abey79/vsvg/pull/71)
- Add input handle hook to the `ViewerApp` trait [#74](https://github.com/abey79/vsvg/pull/74)
- Add `ListItem` UI widget [#75](https://github.com/abey79/vsvg/pull/75)

## Common

- Fit to view on double click [#73](https://github.com/abey79/vsvg/pull/73)
- Add binary publishing support with `cargo-dist` [#78](https://github.com/abey79/vsvg/pull/78)
- Update `CHANGELOG.md` for compatibility with `cargo-dist` and add automation script [#79](https://github.com/abey79/vsvg/pull/79)
- Add plausible.io traffic monitoring to https://whisk.rs [`1228521`](https://github.com/abey79/vsvg/commit/1228521ac97d3286ff2a2f210267a23ee623c969)
- Update to egui 0.24 and wgpu 0.18 [#64](https://github.com/abey79/vsvg/pull/64)

## Contributors

[<img src="https://wsrv.nl/?url=github.com/afternoon2.png?w=64&h=64&mask=circle&fit=cover&maxage=1w" width="32" height="32" alt="afternoon2" />](https://github.com/afternoon2) [<img src="https://wsrv.nl/?url=github.com/karliss.png?w=64&h=64&mask=circle&fit=cover&maxage=1w" width="32" height="32" alt="karliss" />](https://github.com/karliss)


**Full Changelog**: https://github.com/abey79/vsvg/compare/v0.2.0...v0.3.0


# 0.2.0

Released 2023-10-22

## New features

* Add support for Catmull-Rom splines by @abey79 in [#36](https://github.com/abey79/vsvg/pull/36)
* Add some `rng_XXX` convenience functions to `whiskers::Context` and make `rng_range` generic over type by @afternoon2 in [#35](https://github.com/abey79/vsvg/pull/35)
* Add `Grid` helper for grid-based layout by @afternoon2 in [#43](https://github.com/abey79/vsvg/pull/43)
* Add logarithmic slider support to numerical sketch parameter by @abey79 in [#39](https://github.com/abey79/vsvg/pull/39)
* Add support for `vsvg::Color` sketch parameters by @abey79 in [#41](https://github.com/abey79/vsvg/pull/41)
* Add an example to demo the use of the `noise-rs` crate by @abey79 in [#42](https://github.com/abey79/vsvg/pull/42)
* Add in-process profiling with `puffin` and parallelize some layer-level operations by @abey79 in [#44](https://github.com/abey79/vsvg/pull/44)
* Bump egui to 0.23 and wgpu to 0.17 by @abey79 in [#54](https://github.com/abey79/vsvg/pull/54)

## Performance

* Improve performance of `noise` example by @abey79 in [#45](https://github.com/abey79/vsvg/pull/45)
* Refactor `vsvg-viewer` to defer all unneeded render data generation by @abey79 in [#49](https://github.com/abey79/vsvg/pull/49)
* Fix frame profiling order by @abey79 in [#51](https://github.com/abey79/vsvg/pull/51)
* Add tolerance control and vertex count display to `vsvg-viewer` by @abey79 in [#50](https://github.com/abey79/vsvg/pull/50)
* Parallelize native CI jobs by @abey79 in [#37](https://github.com/abey79/vsvg/pull/37)

## Fixes

* Fix README paths linked from main README by @reidab in [#34](https://github.com/abey79/vsvg/pull/34)
* Rename `whiskers::Runner::with_layout_options` for consistency by @abey79 in [#38](https://github.com/abey79/vsvg/pull/38)
* Fix spurious colon in `bool` UI widget label by @abey79 in [#40](https://github.com/abey79/vsvg/pull/40)

## Contributors

[<img src="https://wsrv.nl/?url=github.com/afternoon2.png?w=64&h=64&mask=circle&fit=cover&maxage=1w" width="32" height="32" alt="afternoon2" />](https://github.com/afternoon2) [<img src="https://wsrv.nl/?url=github.com/reidab.png?w=64&h=64&mask=circle&fit=cover&maxage=1w" width="32" height="32" alt="reidab" />](https://github.com/reidab)

**Full Changelog**: https://github.com/abey79/vsvg/compare/v0.1.0...v0.2.0


# 0.1.0

Released 2023-10-01

* Initial release, including:
  * *vsvg*
  * *vsvg-viewer*
  * *vsvg-cli*
  * *whiskers*
  * *whiskers-derive*

  Note: *msvg* is still WIP and not included in this release.
