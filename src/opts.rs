use clap::error::{ContextKind, ContextValue};
use clap::{AppSettings, Parser};
use std::ffi::OsString;
use std::fmt::{self, Display};
use std::str::FromStr;
use syn_select::Selector;

#[derive(Parser)]
#[clap(version)]
pub enum Opts {
    /// Query information from rust
    #[clap(
        name = "whynot",
        version,
        setting = AppSettings::DeriveDisplayOrder,
        subcommand,
    )]
    WhyNot(SubCommand),
    #[clap(external_subcommand, hide = true)]
    Rustc(Vec<OsString>),
}

#[derive(Parser)]
pub enum SubCommand {
    /// Find the reason for why a function is not safe.
    #[clap(
        name = "safe",
        version,
        setting = AppSettings::DeriveDisplayOrder,
        dont_collapse_args_in_usage = true
    )]
    Safe(Args),
}

#[derive(Parser, Debug)]
pub struct Args {
    /// Local path to workspace function to check.
    #[clap(value_name = "ITEM", parse(try_from_str = crate::parse_selector))]
    pub item: Selector,
    #[clap(long, short = 'p')]
    pub package: Option<String>
}

#[derive(Debug, Clone, Copy)]
pub enum Coloring {
    Auto,
    Always,
    Never,
}

impl FromStr for Coloring {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "auto" => Ok(Coloring::Auto),
            "always" => Ok(Coloring::Always),
            "never" => Ok(Coloring::Never),
            other => Err(format!(
                "must be auto, always, or never, but found `{}`",
                other,
            )),
        }
    }
}

impl Display for Coloring {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Coloring::Auto => "auto",
            Coloring::Always => "always",
            Coloring::Never => "never",
        };
        formatter.write_str(name)
    }
}

#[test]
fn test_cli() {
    <Opts as clap::CommandFactory>::command().debug_assert();
}

fn extract(item: (ContextKind, &ContextValue)) -> Option<&ContextValue> {
    let (k, v) = item;
    if k == ContextKind::InvalidArg {
        return Some(v);
    }
    None
}

pub fn parse_known_args() -> Result<(Opts, Vec<String>), eyre::Report> {
    let mut rem: Vec<String> = vec![];
    let mut args: Vec<String> = std::env::args().collect();
    let mut loop_ctr = 100;
    loop {
        loop_ctr -= 1;
        if loop_ctr == 0 {
            Opts::parse();
            break Err(eyre::eyre!(
                "loop overflowed, but parsing was successful, wierd"
            ));
        }
        tracing::debug!("in loop");
        match Opts::try_parse_from(&args) {
            Ok(opts) => {
                break Ok((opts, rem));
            }
            Err(error) => match error.kind() {
                clap::ErrorKind::UnknownArgument => {
                    let items = error.context().find_map(extract);
                    match items {
                        Some(ContextValue::String(s)) => {
                            rem.push(s.to_owned());
                            args.retain(|a| a != s);
                        }
                        _ => {
                            break Err(error.into());
                        }
                    }
                }
                _ => {
                    Opts::parse();
                }
            },
        }
    }
}
