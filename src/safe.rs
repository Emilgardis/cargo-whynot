use std::ffi::OsString;

use eyre::{Context, Result};
use rustc_lint::{LateLintPass, LintContext, LintPass};
use rustc_lint_defs::declare_lint;

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

pub struct UnsafeFindVisitor;

impl LintPass for UnsafeFindVisitor {
    fn name(&self) -> &'static str {
        "unsafe_find_visitor"
    }
}

impl<'tcx> LateLintPass<'tcx> for UnsafeFindVisitor {
    fn check_fn_post(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        _: &'tcx rustc_hir::Body<'tcx>,
        s: rustc_span::Span,
        _: rustc_hir::HirId,
    ) {
        cx.struct_span_lint(UNSAFE_FIND_VISITOR, s, |builder| {
            // use UnsafetyChecker for this.
            builder.build("hello my old friend").emit()
        })
    }
}

declare_lint! {
    /// Custom lint :)
    /// stuff
    UNSAFE_FIND_VISITOR,
    Deny,
    "does stuff"
}

pub struct MyCallback {}

impl rustc_driver::Callbacks for MyCallback {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        let previous = config.register_lints.take();
        config.register_lints = Some(Box::new(move |sess, lint_store| {
            if let Some(previous) = &previous {
                previous(sess, lint_store);
            }

            lint_store.register_late_pass(|| Box::new(UnsafeFindVisitor))
        }))
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
