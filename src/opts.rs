use clap::Parser;
use std::ffi::OsString;
use std::fmt::{self, Display};
use std::str::FromStr;
use syn_select::Selector;

#[derive(Parser, Debug)]
#[clap(version)]
pub enum Opts {
    /// Query information from rust
    #[clap(name = "whynot", version, subcommand)]
    WhyNot(SubCommand),
    #[clap(external_subcommand, hide = true)]
    Rustc(Vec<OsString>),
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    /// Find the reason for why a function is not safe.
    #[clap(name = "safe", version)]
    Safe(Args),
}

#[derive(Parser, Debug)]
pub struct Args {
    /// Local path to workspace function to check.
    #[clap(value_name = "ITEM", value_parser = crate::parse_selector)]
    pub item: Selector,
    #[clap(long, short = 'p')]
    pub package: Option<String>,
    #[clap(default_value = "always")]
    pub color: Coloring,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OutputMode {
    Normal,
    Json,
}

impl FromStr for OutputMode {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(OutputMode::Normal),
            "json" => Ok(OutputMode::Json),
            _ => Err(eyre::eyre!("invalid output mode: {}", s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Coloring {
    Auto,
    Always,
    Never,
}

impl FromStr for Coloring {
    type Err = eyre::Report;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "auto" => Ok(Coloring::Auto),
            "always" => Ok(Coloring::Always),
            "never" => Ok(Coloring::Never),
            other => Err(eyre::eyre!(
                "must be auto, always, or never, but found `{}`",
                other,
            )),
        }
    }
}

impl From<Coloring> for termcolor::ColorChoice {
    fn from(color: Coloring) -> Self {
        match color {
            Coloring::Auto => termcolor::ColorChoice::Auto,
            Coloring::Always => termcolor::ColorChoice::Always,
            Coloring::Never => termcolor::ColorChoice::Never,
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
#[cfg(test)]
fn test_cli() {
    <Opts as clap::CommandFactory>::command().debug_assert();
}
