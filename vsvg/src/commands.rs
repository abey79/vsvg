use crate::cli::{CommandDesc, CommandValue};
use clap::{arg, value_parser, Arg, Id};
use std::collections::HashMap;
use vsvg_core::Transforms;
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
            &|val, doc| match val.try_vector()? {
                [Float(tx), Float(ty)] => Ok(doc.translate(*tx, *ty)),
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
            &|val, doc| {
                if let Float(angle) = val {
                    Ok(doc.rotate(*angle))
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
            &|val, doc| {
                if let Float(angle) = val {
                    Ok(doc.rotate(angle.to_radians()))
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
            &|val, doc| match val.try_vector()? {
                [Float(s)] => Ok(doc.scale(*s)),
                [Float(sx), Float(sy)] => Ok(doc.scale_non_uniform(*sx, *sy)),
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
            &|val, doc| match val.try_vector()? {
                [Float(sx), Float(sy), Float(px), Float(py)] => {
                    Ok(doc.scale_around(*sx, *sy, *px, *py))
                }
                _ => unreachable!("Clap ensure types are correct"),
            },
        ),
        CommandDesc::new(
            arg!(-c --crop [X] "Crop to provided XMIN, YMIN, XMAX, YMAX")
                .value_parser(value_parser!(f64))
                .num_args(4)
                .display_order(order()),
            &|val, doc| match val.try_vector()? {
                [Float(a), Float(b), Float(c), Float(d)] => Ok(doc.crop(*a, *b, *c, *d)),
                _ => unreachable!("Clap ensure types are correct"),
            },
        ),
    ]
    .into_iter()
    .map(|c| (c.id.clone(), c))
    .collect()
}
