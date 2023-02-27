# *vsvg*

## What's this?

This is a fast SVG viewer targeted at plotter users. It is somewhat usable, although it's still a highly experimental WIP. In addition, `vsvg` support a small subset of commands similar to *vpype*'s, albeit with a different syntax (see `vsvg --help`).

<img width="500" alt="image" src="https://user-images.githubusercontent.com/49431240/220178589-e07f7e13-5706-4a7d-bbd4-aefffffa0c58.png">

By "fast", I mean "much faster than [*vpype*](https://github.com/abey79/vpype)". Consider the following command:

```
vpype read 300_beziers.svg show
```

On my computer (a 2021 MacBook Pro M1 Max), *vpype* takes about 570ms *just to start*. Then it needs another 140ms to load this benchmark file containing 300 Bézier curves. Then it kicks off the viewer (this part is actually rather quick).

As of today `vsvg` takes ~3.6ms to start and load the same SVG file. It feels instantaneous.

In another test with a rather pathological 90MB SVG, `vsvg` takes 1.4s to load the file, where it takes upward of 30s for *vpype*. 

## Why?

I've recently started to learn Rust—and loving it so far! 🦀

This project serves me as a training ground and a mean of exploring if/how Rust could be used to power part of [*vpype*](https://github.com/abey79/vpype) and/or [*vsketch*](https://github.com/abey79/vsketch).

`vsvg` already supports features that would be highly beneficial to *vpype*, including:
- a data-model that includes Bézier curve (i.e. there is no loss of accuracy when loading SVG containing curved elements);
- a linearisation process (when one is needed, e.g. for display purposes) with an improved tolerance handling (the segment size adapts based on the curvature, which minimises the number of point needed when curves are nearly straight).

*vpype* is currently made of two packages: `vpype` and `vpype-cli`. The former implements the "engine" and the API, that's then used by `vpype-cli` and plug-ins to offer a CLI interface. What this project explores is basically to opportunity to entirely re-implement the `vpype` package in Rust, which would aptly be renamed `vpype-core` in the process. This would dramatically improve the performance of *vpype*, thanks the Rust being a compiled and making concurrency much easier.

The story for *vsketch* is a bit blurrier at this time. I imagine a complete rewrite using the new, fast `vpype-core`, a dynamic GUI using [`egui`](https://www.egui.rs), while keeping a Python front-end for the user (thanks to `vpype-core` Python bindings needed anyway to interface with `vpype-cli`). That would be lots of work, but the difference in performance would be dramatic, potentially enabling smooth animation and other cool stuff.

This would entail adding `vsk`-like APIs to `vpype-core`. Though this isn't needed for *vpype*, it would potentially enable writing *vpype* plugins in the style of *vsketch*, which sounds like a cool idea to me.

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

Build and install `vsvg`:

```
cargo build --release
cargo install --path .
```

(If you cannot run `cargo`, your path hasn't been set correctly when installing Rust. Restart your terminal and try again. If it still fails, add `$HOME/.cargo/bin` in your PATH.)

`vsvg` should now be available:

```
vsvg --help
```

To uninstall `vsvg`, navigate back to the `vsvg` source directory and execute the following command:

```
cargo uninstall
```


## Design notes

Here are a few design considerations, in the frame of using this project as basis for a future, Rust-based `vpype-core` project.


### Elementary path

*vpype* uses one-dimensional Numpy array of `complex` as basic path type. This means that anything curvy must be linearised and transformed into a polyline. This is why the `read` command has a `-q/--quantization` option to control the accuracy of this transformation. One design goal of *vpype* 2 is to no longer degrade curved paths into polylines.

Another design goal is to support compound paths, i.e. paths made of several, possibly-closing sub-paths. This is used to represent shapes with holes. Proper support for shapes with holes is a strong prerequisite towards a robust hatch filling feature for *vpype*.

One approach is to support *all* of SVG primitives: elliptic arcs (including full circles and ellipses), quadratic Beziers, and cubic Bézier. The drawback is the added complexity of dealing with so many primitives. Another approach would be to support only polylines and cubic Bézier. They provide an exact approximation of quadratic bezier and a good approximation of arcs, while generally be nice to work with.

As it turns out, the `usvg` crate offers facilities to normalise any SVG primitive into paths made of polylines and cubic Bézier only. Its output relies on the `BezPath` structure from the `kurbo` crate, which offers a dual representation: draw commands (MoveTo, LineTo, CurveTo, ClosePath) and segments (LineSegment, BezierSegment). Each representation has advantages in different circumstances. Conveniently, `BezPath` also supports compound paths.

Consequently, my current plan is to use `krubo::BezPath` as fundamental structure for path representation, and build a Path/Layer/Document hierarchy around it.


### Flattened paths

The case for fully "flattened" representations (e.g. representations where everything is converted to polyline) remains. At the very least, this will be needed for the viewer (unless Bezier -> polyline can be figured out in a shader). HPGL and gcode export also come to mind. Finally, backward plug-in compatibility could also benefit from that, although this is not really a design goal. When flattening a document, keeping the Doc/Layer/Path hierarchy would still be needed though, i.e. for the viewer to handle layer visibility or properly colouring each path using its attached metadata.

To minimise code duplication, my plan is use the following data structures:


```
        CONCRETE                     GENERIC                    FLATTENED        
     IMPLEMENTATION              IMPLEMENTATION              IMPLEMENTATION      
                                                                                 
+-----------------------+   +-----------------------+   +-----------------------+
|         Path          |   |                       |   |     FlattenedPath     |
|                       |<--|      PathImpl<T>      |-->|                       |
|   PathImpl<BezPath>   |   |                       |   |  PathImpl<Polyline>   |
+-----------------------+   +-----------------------+   +-----------------------+
                                                                                 
+-----------------------+   +-----------------------+   +-----------------------+
|         Layer         |   |                       |   |    FlattenedLayer     |
|                       |<--|     LayerImpl<T>      |-->|                       |
|  LayerImpl<BezPath>   |   |                       |   |  LayerImpl<Polyline>  |
+-----------------------+   +-----------------------+   +-----------------------+
                                                                                 
+-----------------------+   +-----------------------+   +-----------------------+
|       Document        |   |                       |   |   FlattenedDocument   |
|                       |<--|    DocumentImpl<T>    |-->|                       |
| DocumentImpl<BezPath> |   |                       |   |DocumentImpl<Polyline> |
+-----------------------+   +-----------------------+   +-----------------------+
                                                                                 
                                                         Polyline = Vec<[f64; 2]>
```

The core of the Path/Layer/Document hierarchy is implemented with structures that are generic over the actual path data type. Then, two sets of concrete types are offered, one based on `BezPath` and another based on a simple vector of points. Any feature that is easy to implement generically is done in `XXXImpl<T>`. Features that would require too much work to cover both hierarchies and not strictly necessary for the flattened use cases are implemented for `Path` and friends only. Conversion between normal and flattened structure is offered, though obviously the round-trip would be destructive.


### Mutability of Path/Layer/Doc

For operations on Path/Layer/Document structures (e.g. transforms, crop, etc.), I'm using for the moment an immutable pattern (e.g. `fn ops(self, ...) -> Self {}`), though clearly the goal is not to have purely immutable structure (e.g. paths addition/removal in layers, layer addition/removal in document, etc.). I'm not sure yet of the implication and if this is a good idea or not.


### Metadata handling

I'm still in the process of sorting that out.

There are at least two design goals:
- The data structure should support the hierarchical nature of metadata, i.e. color is looked up for a path but not defined, the look-up should escalate to the layer, then to the document, then to default values.
- Cloning metadata should be cheap. For example, flattening a document should not copy all the metadata upon cloning, but only lazily upon mutation—if any.

Maybe a `HashMap<_, Cow<_>>`? Or immutable data structure from the `im` crate?

## TODO

- [x] Sort out page orientation and check that rotation, etc. work the same as with vpype
- [ ] ~~egui plot viewer cannot display zoom-aware fat lines :(~~ I'll deal with the viewer at a later stage—vpype 2 could keep the existing viewer. 
- [ ] ~~Properly handle Y axis (currently it's flipped)~~ (probably pointless if we move to a custom viewer)
  - [ ] ~~Custom y_axis_formatter~~
- [x] Add support for color and line width (but width is not zoom-aware)
- [x] Crop to page size
- [ ] ~~Test viewbox~~ Fix viewbox handling
- [ ] Fix missing top-level paths
- [ ] Metadata concept, possibly using `Rc`'s clone-on-write capability
- [x] Split types.rs into multiple files (e.g. `types/document.rs`, `types/layer.rs`, etc.)
- [x] Move stuff to `lib.rs`
- [ ] Implement *vpype*-like layer IDs.
- [ ] Rename `Path` to `Shape` to denote it being higher level?
- [ ] Implement some Drawer API + add related commands? 
- [ ] .......
