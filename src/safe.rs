use std::{collections::VecDeque, ffi::OsString, sync::Arc};

use eyre::{Context, Result};
use hir::{def_id::LocalDefId, ItemId};
use rustc_codegen_ssa::{traits::CodegenBackend, CodegenResults};
use rustc_driver::Callbacks;
use rustc_hash::FxHashMap;
use rustc_hir as hir;
use rustc_middle::{
    dep_graph::{WorkProduct, WorkProductId},
    mir::{self, UnsafetyViolation, UnsafetyViolationDetails, UnsafetyViolationKind},
    ty::{self, query::ExternProviders},
};

pub mod checker;
pub mod types;

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
    let selector = std::env::var(crate::ENV_VAR_WHYNOT_SELECTOR)
        .wrap_err(crate::WHYNOT_RUSTC_WRAPPER_ERROR)?;

    let _ = crate::parse_selector(&selector)?;
    tracing::debug!("in whynot safe rustc with selector: {selector:?}");
    tracing::debug!("in whynot safe rustc with rem: `{rem:?}`");

    crate::run::rustc_run(
        None,
        Some(Box::new(move |_| Box::new(FakeCodeGen { selector }))),
        &rem[1..],
    )?;

    Ok(())
}

pub struct FakeCodeGen {
    selector: String,
}

impl FakeCodeGen {
    pub fn selector(&self) -> syn_select::Selector {
        crate::parse_selector(&self.selector).expect("this should not fail")
    }

    pub fn run<'tcx>(&self, tcx: ty::TyCtxt<'tcx>) -> Result<()> {
        let fun_id = self.search(tcx)?;
        let reason = self.find_unsafe_things(tcx, fun_id)?;
        tracing::info!("reason = {:?}", reason);
        Ok(())
    }

    pub fn find_unsafe_things<'tcx>(
        &self,
        tcx: ty::TyCtxt<'tcx>,
        fun_id: ItemId,
    ) -> Result<Vec<UnsafetyViolation>> {
        let mut reasons = vec![];
        let mut possible_reasons = vec![].into();
        let mut did = fun_id.def_id;
        let mut loopctr = 10;
        loop {
            loopctr -= 1;
            if loopctr == 0 {
                break;
            }
            self.find_unsafe_things_(tcx, did, &mut reasons, &mut possible_reasons, &mut 100)?;
            if reasons.is_empty() {
                // take the first possible unsafe fn, if it is local, check why it is unsafe.
                'possible: while let Some(violation) = possible_reasons.pop_front() {
                    match violation.kind {
                        UnsafetyViolationKind::UnsafeFn => {
                            todo!(
                                "find the functions definition, and then check why it is unsafe."
                            );
                            self.find_unsafe_things_(
                                tcx,
                                did,
                                &mut reasons,
                                &mut possible_reasons,
                                &mut 100,
                            )?;
                            break 'possible;
                        }
                        _ => continue 'possible,
                    }
                }
            } else {
                break;
            }
        }
        Ok(reasons)
    }

    fn find_unsafe_things_<'tcx>(
        &self,
        tcx: ty::TyCtxt<'tcx>,
        def_id: LocalDefId,
        reasons: &mut Vec<UnsafetyViolation>,
        possible_reasons: &mut VecDeque<UnsafetyViolation>,
        depth: &mut i32,
    ) -> Result<(), color_eyre::Report> {
        *depth -= 1;
        if *depth == 0 {
            panic!()
        }
        let res: &'_ mir::UnsafetyCheckResult = tcx.unsafety_check_result(def_id);
        dbg!(res);
        for violation in &res.violations {
            match (violation.kind, violation.details) {
                (
                    UnsafetyViolationKind::UnsafeFn,
                    UnsafetyViolationDetails::CallToUnsafeFunction,
                ) => {
                    possible_reasons.push_back(violation.clone());
                }
                _ => reasons.push(violation.clone()),
            }
        }
        Ok(())
    }

    /// Search for the selectors itemid
    pub fn search<'tcx>(&self, tcx: ty::TyCtxt<'tcx>) -> Result<ItemId> {
        // Find def_id of function in selector.
        let hir = tcx.hir();
        hir.items()
            .find(|item| {
                if tcx.def_kind(item.def_id.to_def_id()) != hir::def::DefKind::Fn {
                    return false;
                }
                let path = tcx.def_path(item.def_id.to_def_id());
                tracing::debug!(no_crate = ?path.to_string_no_crate_verbose());
                tracing::debug!(debug = ?path);

                if path.to_string_no_crate_verbose().ends_with(&self.selector) {
                    return true;
                }
                false
            })
            .ok_or(eyre::eyre!("no such function found"))
    }
}

impl CodegenBackend for FakeCodeGen {
    fn codegen_crate<'tcx>(
        &self,
        tcx: ty::TyCtxt<'tcx>,
        metadata: rustc_metadata::EncodedMetadata,
        need_metadata_module: bool,
    ) -> Box<dyn std::any::Any> {
        tracing::info!("codegen_crate");
        match self.run(tcx) {
            Err(e) => tracing::error!("{}", e),
            Ok(_) => (),
        };
        Box::new(())
    }

    fn join_codegen(
        &self,
        _: Box<dyn std::any::Any>,
        _: &rustc_session::Session,
        _: &rustc_session::config::OutputFilenames,
    ) -> Result<
        (
            rustc_codegen_ssa::CodegenResults,
            rustc_hash::FxHashMap<
                rustc_middle::dep_graph::WorkProductId,
                rustc_middle::dep_graph::WorkProduct,
            >,
        ),
        rustc_errors::ErrorGuaranteed,
    > {
        std::process::exit(0);
    }

    fn link(
        &self,
        _: &rustc_session::Session,
        _: rustc_codegen_ssa::CodegenResults,
        _: &rustc_session::config::OutputFilenames,
    ) -> Result<(), rustc_errors::ErrorGuaranteed> {
        unimplemented!()
    }
}
