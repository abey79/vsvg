use crate::cli::{CommandDesc, CommandValue};
use crate::types::transforms::Transforms;
use clap::{arg, value_parser, Id};
use std::collections::HashMap;

pub(crate) fn command_list() -> HashMap<Id, CommandDesc<'static>> {
    [
        CommandDesc::new(
            arg!(-t --translate [X] "Translate")
                .value_parser(value_parser!(f64))
                .num_args(1..=2),
            &|val, doc| {
                if let CommandValue::Vector(v) = val {
                    match v[..] {
                        [CommandValue::Float(t)] => doc.translate(t, t),
                        [CommandValue::Float(tx), CommandValue::Float(ty)] => doc.translate(tx, ty),
                        _ => unreachable!("Clap ensure types are correct"),
                    }
                } else {
                    unimplemented!("Clap ensure types are correct");
                }
            },
        ),
        CommandDesc::new(
            arg!(-r --rotate [X] "Rotate")
                .value_parser(value_parser!(f64))
                .num_args(1..=2),
            &|_val, doc| doc,
        ),
    ]
    .into_iter()
    .map(|c| (c.id.clone(), c))
    .collect()
}
