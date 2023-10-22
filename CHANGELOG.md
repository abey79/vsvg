# Change log

## 0.2.0 (2023-10-22)

### New features

* Add support for Catmull-Rom splines by @abey79 in [#36](https://github.com/abey79/vsvg/pull/36)
* Add some `rng_XXX` convenience functions to `whiskers::Context` and make `rng_range` generic over type by @afternoon2 in [#35](https://github.com/abey79/vsvg/pull/35)
* Add `Grid` helper for grid-based layout by @afternoon2 in [#43](https://github.com/abey79/vsvg/pull/43)
* Add logarithmic slider support to numerical sketch parameter by @abey79 in [#39](https://github.com/abey79/vsvg/pull/39)
* Add support for `vsvg::Color` sketch parameters by @abey79 in [#41](https://github.com/abey79/vsvg/pull/41)
* Add an example to demo the use of the `noise-rs` crate by @abey79 in [#42](https://github.com/abey79/vsvg/pull/42)
* Add in-process profiling with `puffin` and parallelize some layer-level operations by @abey79 in [#44](https://github.com/abey79/vsvg/pull/44)
* Bump egui to 0.23 and wgpu to 0.17 by @abey79 in [#54](https://github.com/abey79/vsvg/pull/54)

### Performance

* Improve performance of `noise` example by @abey79 in [#45](https://github.com/abey79/vsvg/pull/45)
* Refactor `vsvg-viewer` to defer all unneeded render data generation by @abey79 in [#49](https://github.com/abey79/vsvg/pull/49)
* Fix frame profiling order by @abey79 in [#51](https://github.com/abey79/vsvg/pull/51)
* Add tolerance control and vertex count display to `vsvg-viewer` by @abey79 in [#50](https://github.com/abey79/vsvg/pull/50)
* Parallelize native CI jobs by @abey79 in [#37](https://github.com/abey79/vsvg/pull/37)

### Fixes

* Fix README paths linked from main README by @reidab in [#34](https://github.com/abey79/vsvg/pull/34)
* Rename `whiskers::Runner::with_layout_options` for consistency by @abey79 in [#38](https://github.com/abey79/vsvg/pull/38)
* Fix spurious colon in `bool` UI widget label by @abey79 in [#40](https://github.com/abey79/vsvg/pull/40)

### New Contributors
* @reidab made their first contribution in [#34](https://github.com/abey79/vsvg/pull/34)

**Full Changelog**: https://github.com/abey79/vsvg/compare/v0.1.0...v0.2.0


## 0.1.0 (2023-10-01)

* Initial release, including:
  * *vsvg*
  * *vsvg-viewer*
  * *vsvg-cli*
  * *whiskers*
  * *whiskers-derive*

  Note: *msvg* is still WIP and not included in this release.
