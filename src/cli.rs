use clap::{arg, command, value_parser, Arg, ArgAction, ArgGroup, ArgMatches, Command, Id};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;

use std::fmt::{Debug, Display, Formatter};

use crate::types::Document;
use std::path::PathBuf;

/// A trait for types that can be used as command line arguments.
trait CommandArg: Clone + Into<CommandValue> + Send + Sync + Debug + 'static {}
impl<T: Clone + Into<CommandValue> + Send + Sync + Debug + 'static> CommandArg for T {}

#[derive(Debug)]
pub enum CliError {
    NotAVector,
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::NotAVector => write!(f, "Not a vector"),
        }
    }
}

impl Error for CliError {}

trait ArgCreator {
    type T: CommandArg;

    fn create_arg(&self, arg: &mut Arg);
}

pub(crate) struct CommandDesc<'a> {
    pub(crate) id: Id,
    pub(crate) arg: Arg,
    pub(crate) multiple_args: bool,
    pub(crate) action: &'a dyn Fn(&CommandValue, Document) -> Result<Document, Box<dyn Error>>,
}

impl<'a> CommandDesc<'a> {
    pub(crate) fn new(
        arg: Arg,
        action: &'a dyn Fn(&CommandValue, Document) -> Result<Document, Box<dyn Error>>,
    ) -> Self {
        let multiple_args = arg.get_num_args().unwrap_or_default().max_values() > 1;
        Self {
            id: arg.get_id().clone(),
            arg: arg.group("commands").action(ArgAction::Append),
            multiple_args,
            action,
        }
    }
}

pub(crate) fn cli(command_descs: &HashMap<Id, CommandDesc>) -> Command {
    let mut cli = command!()
        .args([
            arg!(<PATH> "Path to the SVG file").value_parser(value_parser!(PathBuf)),
            Arg::new("no-show")
                .long("no-show")
                .help("Don't show the GUI")
                .num_args(0),
            arg!(-v --verbose "Enable debug output"),
        ])
        .group(ArgGroup::new("commands").multiple(true))
        .next_help_heading("COMMANDS");

    for command in command_descs.values() {
        cli = cli.arg(command.arg.clone());
    }

    cli
}

#[derive(Clone, PartialEq, Debug)]
pub enum CommandValue {
    Bool(bool),
    String(String),
    Float(f64),
    Vector(Vec<CommandValue>),
}

impl CommandValue {
    pub(crate) fn from_matches(
        matches: &ArgMatches,
        command_descs: &HashMap<Id, CommandDesc>,
    ) -> Vec<(Id, Self)> {
        let mut values = BTreeMap::new();
        for id in matches.ids() {
            if matches.try_get_many::<Id>(id.as_str()).is_ok() {
                // ignore groups
                continue;
            }
            let value_source = matches
                .value_source(id.as_str())
                .expect("id came from matches");
            if value_source != clap::parser::ValueSource::CommandLine {
                // Any other source just gets tacked on at the end (like default values)
                continue;
            }

            let desc = command_descs.get(id).expect("id came from matches");

            if Self::extract::<String>(matches, id, desc, &mut values) {
                continue;
            }
            if Self::extract::<bool>(matches, id, desc, &mut values) {
                continue;
            }
            if Self::extract::<f64>(matches, id, desc, &mut values) {
                continue;
            }
            unimplemented!("unknown type for {}: {:?}", id, matches);
        }
        values.into_values().collect::<Vec<_>>()
    }

    fn extract<T: CommandArg>(
        matches: &ArgMatches,
        id: &Id,
        command_desc: &CommandDesc,
        output: &mut BTreeMap<usize, (Id, Self)>,
    ) -> bool {
        match matches.try_get_many::<T>(id.as_str()) {
            Ok(Some(_)) => {
                let occurrences: Vec<Vec<T>> = matches
                    .get_occurrences(id.as_str())
                    .expect("id came from matches")
                    .map(|occ| occ.cloned().collect())
                    .collect();

                let indices: Vec<usize> = matches
                    .indices_of(id.as_str())
                    .expect("id came from matches")
                    .collect();

                let mut indices_idx = 0_usize;
                for value in occurrences.iter() {
                    let index = indices[indices_idx];

                    if command_desc.multiple_args {
                        output.insert(index, (id.clone(), value.clone().into()));
                    } else {
                        output.insert(index, (id.clone(), value[0].clone().into()));
                    }

                    indices_idx += value.len();
                }

                true
            }
            Ok(None) => {
                unreachable!("`ids` only reports what is present")
            }
            Err(clap::parser::MatchesError::UnknownArgument { .. }) => {
                unreachable!("id came from matches")
            }
            Err(clap::parser::MatchesError::Downcast { .. }) => false,
            Err(_) => {
                unreachable!("id came from matches")
            }
        }
    }

    pub(crate) fn try_vector(&self) -> Result<&[CommandValue], CliError> {
        match self {
            Self::Vector(v) => Ok(&v[..]),
            _ => Err(CliError::NotAVector),
        }
    }
}

impl From<String> for CommandValue {
    fn from(other: String) -> Self {
        Self::String(other)
    }
}

impl From<bool> for CommandValue {
    fn from(other: bool) -> Self {
        Self::Bool(other)
    }
}

impl From<f64> for CommandValue {
    fn from(other: f64) -> Self {
        Self::Float(other)
    }
}

impl<T: CommandArg> From<Vec<T>> for CommandValue {
    fn from(other: Vec<T>) -> Self {
        Self::Vector(other.into_iter().map(|v| v.into()).collect())
    }
}
