use std::ffi::OsString;

use eyre::{Context, Result};

use crate::run::cargo_check;

pub(crate) fn run(args: crate::opts::Args, rem: &[String]) -> Result<()> {
    tracing::debug!("checking");
    cargo_check("safe", Some(args.item.to_string()), rem)
}

pub(crate) fn run_rustc(rem: &[OsString]) -> Result<()> {
    assert_eq!(
        std::env::var(crate::ENV_VAR_WHYNOT_MODE).as_deref(),
        Ok("safe")
    );
    let selector = crate::parse_selector(
        &std::env::var(crate::ENV_VAR_WHYNOT_SELECTOR)
            .wrap_err(crate::WHYNOT_RUSTC_WRAPPER_ERROR)?,
    )?;
    tracing::debug!("in whynot safe rustc with selector: {selector:?}");
    tracing::debug!("in whynot safe rustc with rem: `{rem:?}`");

    crate::run::rustc_run(&mut MyCallback {}, &rem[1..])?;
    Ok(())
}

pub struct MyCallback {}

impl rustc_driver::Callbacks for MyCallback {
    fn config(&mut self, _config: &mut rustc_interface::interface::Config) {
        // fixme: this is where it all happens (well, not really but still)
    }

    fn after_parsing<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        rustc_driver::Compilation::Continue
    }

    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        rustc_driver::Compilation::Continue
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> rustc_driver::Compilation {
        rustc_driver::Compilation::Stop
    }
}
