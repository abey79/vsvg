# SVGER

## TODO

- [ ] egui plot viewer cannot display zoom-aware fat lines :(
- [ ] ~~Properly handle Y axis (currently it's flipped)~~ (probably pointless if we move to a custom viewer)
  - [ ] ~~Custom y_axis_formatter~~
- [x] Add support for color and line width (but width is zoom-aware)
- [x] Crop to page size
- [ ] Test viewbox
- [ ] Metadata concept, possibly using `Rc`'s clone-on-write capability
- [ ] Split types.rs into multiple files (e.g. `types/document.rs`, `types/layer.rs`, etc.)
- [ ] Move stuff to `lib.rs`
- [ ] Implement *vpype*-like layer IDs.