# Unreleased

TODO

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
