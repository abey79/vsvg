use crate::cli::{CommandDesc, CommandValue};
use clap::{arg, value_parser, Arg, Id};
use std::collections::HashMap;
use vsvg_core::{LayerID, Transforms};
use CommandValue::Float;

pub(crate) fn command_list() -> HashMap<Id, CommandDesc<'static>> {
    // small utility to generate an increasing order for each command
    let mut order_cnt = 0_usize;
    let mut order = || -> usize {
        order_cnt += 1;
        order_cnt
    };

    [
        CommandDesc::new(
            arg!(-t --translate [X] "Translate by provided coordinates")
                .value_parser(value_parser!(f64))
                .num_args(2)
                .display_order(order()),
            &|val, state| match val.try_vector()? {
                [Float(tx), Float(ty)] => {
                    state.document.translate(*tx, *ty);
                    Ok(())
                }
                _ => unreachable!("Clap ensure types are correct"),
            },
        ),
        CommandDesc::new(
            Arg::new("rotate-rad")
                .short('R')
                .long("rotate-rad")
                .value_name("X")
                .help("Rotate by X radians around the origin")
                .value_parser(value_parser!(f64))
                .num_args(1)
                .display_order(order()),
            &|val, state| {
                if let Float(angle) = val {
                    state.document.rotate(*angle);
                    Ok(())
                } else {
                    unreachable!("Clap ensure types are correct")
                }
            },
        ),
        CommandDesc::new(
            arg!(-r --rotate [X] "Rotate by X degrees around the origin")
                .value_parser(value_parser!(f64))
                .num_args(1)
                .display_order(order()),
            &|val, state| {
                if let Float(angle) = val {
                    state.document.rotate(angle.to_radians());
                    Ok(())
                } else {
                    unreachable!("Clap ensure types are correct")
                }
            },
        ),
        CommandDesc::new(
            arg!(-s --scale [X] "Uniform (X) or non-uniform (X Y) scaling around the origin")
                .value_parser(value_parser!(f64))
                .num_args(1..=2)
                .display_order(order()),
            &|val, state| match val.try_vector()? {
                [Float(s)] => {
                    state.document.scale(*s);
                    Ok(())
                }
                [Float(sx), Float(sy)] => {
                    state.document.scale_non_uniform(*sx, *sy);
                    Ok(())
                }
                _ => unreachable!("Clap ensure types are correct"),
            },
        ),
        CommandDesc::new(
            Arg::new("scale-around")
                .long("scale-around")
                .value_name("X")
                .help("Scale around the provided point")
                .value_parser(value_parser!(f64))
                .num_args(4)
                .display_order(order()),
            &|val, state| match val.try_vector()? {
                [Float(sx), Float(sy), Float(px), Float(py)] => {
                    state.document.scale_around(*sx, *sy, *px, *py);
                    Ok(())
                }
                _ => unreachable!("Clap ensure types are correct"),
            },
        ),
        CommandDesc::new(
            arg!(-c --crop [X] "Crop to provided XMIN, YMIN, XMAX, YMAX")
                .value_parser(value_parser!(f64))
                .num_args(4)
                .display_order(order()),
            &|val, state| match val.try_vector()? {
                [Float(a), Float(b), Float(c), Float(d)] => {
                    state.document.crop(*a, *b, *c, *d);
                    Ok(())
                }
                _ => unreachable!("Clap ensure types are correct"),
            },
        ),
        CommandDesc::new(
            arg!(--dlayer [X] "Set target layer for draw operations")
                .value_parser(value_parser!(LayerID))
                .num_args(1)
                .display_order(order()),
            &|val, state| {
                if let CommandValue::LayerID(lid) = val {
                    state.draw_layer = *lid;
                    Ok(())
                } else {
                    unreachable!("Clap ensure types are correct")
                }
            },
        ),
        CommandDesc::new(
            arg!(--dtranslate [X] "Apply an X, Y translation to the current transform")
                .value_parser(value_parser!(f64))
                .num_args(2)
                .display_order(order()),
            &|val, state| match val.try_vector()? {
                [Float(dx), Float(dy)] => {
                    state.draw_state.translate(*dx, *dy);

                    Ok(())
                }
                _ => unreachable!("Clap ensure types are correct"),
            },
        ),
        CommandDesc::new(
            arg!(--drotate [X] "Apply a rotation to the current transform")
                .value_parser(value_parser!(f64))
                .num_args(1)
                .display_order(order()),
            &|val, state| {
                if let Float(angle) = val {
                    state.draw_state.rotate(angle.to_radians());
                    Ok(())
                } else {
                    unreachable!("Clap ensure types are correct")
                }
            },
        ),
        CommandDesc::new(
            arg!(--drect [X] "Draw a rectangle with X, Y, W, H")
                .value_parser(value_parser!(f64))
                .num_args(4)
                .display_order(order()),
            &|val, state| match val.try_vector()? {
                [Float(a), Float(b), Float(c), Float(d)] => {
                    state
                        .document
                        .get_mut(state.draw_layer)
                        .draw(&state.draw_state)
                        .rect(*a, *b, *c, *d);

                    Ok(())
                }
                _ => unreachable!("Clap ensure types are correct"),
            },
        ),
    ]
    .into_iter()
    .map(|c| (c.id.clone(), c))
    .collect()
}
