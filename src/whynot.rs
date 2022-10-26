#![feature(rustc_private)]
#![feature(let_chains, box_patterns)]

extern crate rustc_codegen_ssa;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

pub static ENV_VAR_WHYNOT_MODE: &str = "__CARGO-WHYNOT_MODE";
pub static ENV_VAR_WHYNOT_COLORING: &str = "__CARGO-WHYNOT_COLORING";
pub static ENV_VAR_WHYNOT_SELECTOR: &str = "__CARGO-WHYNOT_SELECTOR";
pub static WHYNOT_RUSTC_WRAPPER_ERROR: &str = "ran `cargo whynot rustc` outside of wrapper";

mod opts;
mod run;
mod safe;
mod utils;
use std::str::FromStr;

use clap::Parser;
use syn_select::Selector;

use self::opts::{Opts, SubCommand};

fn main() -> eyre::Result<()> {
    let command = Opts::parse();
    tracing::debug!("command: {command:?}");
    match command {
        Opts::WhyNot(sc) => {
            utils::install_utils()?;
            match sc {
                SubCommand::Safe(args) => safe::run(args, &[])?,
            }
        }
        Opts::Rustc(external) => match std::env::var(ENV_VAR_WHYNOT_MODE).as_deref() {
            Ok("safe") => safe::run_rustc(&external)?,
            _ => eyre::bail!(WHYNOT_RUSTC_WRAPPER_ERROR),
        },
    }

    Ok(())
}

pub fn parse_selector(s: &str) -> Result<Selector, <Selector as FromStr>::Err> {
    if let Some(stripped) = s.strip_prefix("::") {
        stripped.parse()
    } else if let Some(stripped) = s.strip_prefix("crate::") {
        stripped.parse()
    } else {
        s.parse()
    }
}
