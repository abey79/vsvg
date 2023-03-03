use crate::cli::{CommandDesc, CommandValue};
use clap::{arg, value_parser, Arg, Id};
use std::collections::HashMap;
use vsvg_core::Transforms;

// https://stackoverflow.com/a/38361018/229511
macro_rules! count_items {
    ($name:ident) => { 1 };
    ($first:ident, $($rest:ident),*) => {
        1 + count_items!($($rest),*)
    }
}

// copied from min_max crate
macro_rules! min {
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) => {
        std::cmp::min($x, min!( $($xs),+ ))
    };
}

macro_rules! max {
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) => {
        std::cmp::max($x, max!( $($xs),+ ))
    };
}

macro_rules! first_ident {
    ($x:ident) => {
        $x
    };
    ($x:ident, $($xs:ident),+) => {
        $x
    };
}

macro_rules! command_impl {
    ($arg:expr, $t1:ty, $t2:ident, |$state:ident, $x:ident| $action:expr) => {
        CommandDesc::new(
            $arg.value_parser(value_parser!($t1)).num_args(1).display_order(order()),
            &|val, $state| {
                if let CommandValue::$t2($x) = val {
                    $action;
                    Ok(())
                } else {
                    unreachable!("Clap ensure types are correct")
                }
            },
        )
    };
    ($arg:expr, $t1:ty, $t2:ident, $(|$state:ident, $($x:ident),+| $action:expr),+) => {
        CommandDesc::new(
            $arg
                .value_parser(value_parser!($t1))
                .num_args(min!($(count_items!($($x),+)),+)..=max!($(count_items!($($x),+)),+))
                .display_order(order()),
            &|val, first_ident!($($state),+)| match val.try_vector()? {
                $([$(CommandValue::$t2($x)),+] => {
                    $action;
                    Ok(())
                }),+
                _ => unreachable!("Clap ensure types are correct"),
            },
        )
    };
}

macro_rules! command_decl {
    ($arg:expr, f64, $(|$state:ident, $($x:ident),+| $action:expr),+) => {
        command_impl!($arg, f64, Float, $(|$state, $($x),+| $action),+)
    };
    ($arg:expr, bool, $(|$state:ident, $($x:ident),+| $action:expr),+) => {
        command_impl!($arg, bool, Bool, $(|$state, $($x),+| $action),+)
    };
    ($arg:expr, String, $(|$state:ident, $($x:ident),+| $action:expr),+) => {
        command_impl!($arg, String, String, $(|$state, $($x),+| $action),+)
    };
    ($arg:expr, LayerID, $(|$state:ident, $($x:ident),+| $action:expr),+) => {
        command_impl!($arg, vsvg_core::LayerID, LayerID, $(|$state, $($x),+| $action),+)
    };
}

// this needs to be implemented this way such as to be available from the macros
fn order() -> usize {
    static mut ORDER: usize = 0;
    unsafe {
        ORDER += 1;
        ORDER
    }
}

pub(crate) fn command_list() -> HashMap<Id, CommandDesc<'static>> {
    [
        command_decl!(
            arg!(-t --translate [X] "Translate by provided coordinates"),
            f64,
            |state, tx, ty| state.document.translate(*tx, *ty)
        ),
        command_decl!(
            Arg::new("rotate-rad")
                .short('R')
                .long("rotate-rad")
                .value_name("X")
                .help("Rotate by X radians around the origin"),
            f64,
            |state, angle| state.document.rotate(*angle)
        ),
        command_decl!(
            arg!(-r --rotate [X] "Rotate by X degrees around the origin"),
            f64,
            |state, angle| state.document.rotate(angle.to_radians())
        ),
        command_decl!(
            arg!(-s --scale [X] "Uniform (X) or non-uniform (X Y) scaling around the origin"),
            f64,
            |state, s| state.document.scale(*s),
            |state, sx, sy| state.document.scale_non_uniform(*sx, *sy)
        ),
        command_decl!(
            Arg::new("scale-around")
                .long("scale-around")
                .value_name("X")
                .help("Scale around the provided point"),
            f64,
            |state, sx, sy, px, py| state.document.scale_around(*sx, *sy, *px, *py)
        ),
        command_decl!(
            arg!(-c --crop [X] "Crop to provided XMIN, YMIN, XMAX, YMAX"),
            f64,
            |state, a, b, c, d| state.document.crop(*a, *b, *c, *d)
        ),
        command_decl!(
            arg!(--dlayer [X] "Set target layer for draw operations"),
            LayerID,
            |state, lid| state.draw_layer = *lid
        ),
        command_decl!(
            arg!(--dtranslate [X] "Apply an X, Y translation to the current transform"),
            f64,
            |state, dx, dy| state.draw_state.translate(*dx, *dy)
        ),
        command_decl!(
            arg!(--drotate [X] "Apply a rotation to the current transform"),
            f64,
            |state, angle| state.draw_state.rotate(angle.to_radians())
        ),
        command_decl!(
            arg!(--dscale [X] "Apply a uniform (X) or non-uniform (X, Y) scale to the current transform"),
            f64,
            |state, s| state.draw_state.scale(*s),
            |state, sx, sy| state.draw_state.scale_non_uniform(*sx, *sy)
        ),
        command_decl!(
            arg!(--drect [X] "Draw a rectangle with X, Y, W, H"),
            f64,
            |state, a, b, c, d| {
                state.document.get_mut(state.draw_layer).draw(&state.draw_state).rect(*a, *b, *c, *d)
            }
        ),
    ]
    .into_iter()
    .map(|c| (c.id.clone(), c))
    .collect()
}
