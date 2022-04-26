pub mod expr;
pub mod unsafety_visitor;

use std::{collections::VecDeque, ffi::OsString};

use eyre::{Context, Result};
use hir::{def_id::LocalDefId, itemlikevisit::ItemLikeVisitor, FnHeader};
use rustc_hir as hir;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;

use crate::{run::cargo_check, safe::unsafety_visitor::UnsafeOpKind};

pub(crate) fn run(args: crate::opts::Args, rem: &[String]) -> Result<()> {
    tracing::debug!("checking");
    cargo_check(
        "safe",
        Some(args.item.to_string()),
        &args.package,
        Some("-Zthir-unsafeck"),
        rem,
    )
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

    crate::run::rustc_run(Some(&mut FakeCallback { selector }), None, &rem[1..])?;

    Ok(())
}

pub struct FakeCallback {
    selector: String,
}

impl FakeCallback {
    pub fn run<'tcx>(&self, tcx: ty::TyCtxt<'tcx>) -> Result<()> {
        let (fun_id, header) = self.search(tcx)?;
        tracing::debug!(?header);
        if header.unsafety == hir::Unsafety::Normal {
            println!("function is not unsafe");
            return Ok(());
        }
        let reason = self.find_unsafe_things(tcx, fun_id)?;
        println!("reason = {:?}", reason);
        Ok(())
    }

    pub fn find_unsafe_things<'tcx>(
        &self,
        tcx: ty::TyCtxt<'tcx>,
        fun_id: LocalDefId,
    ) -> Result<Vec<(UnsafeOpKind, Span)>> {
        let mut reasons = vec![];
        let mut possible_reasons = vec![].into();
        let mut unsafe_fns_in_cur = vec![];
        let mut did = fun_id;
        let mut loopctr = 10;
        let mut first_loop = true;
        loop {
            loopctr -= 1;
            if loopctr == 0 {
                break;
            }

            self.find_unsafe_things_(tcx, did, &mut reasons, &mut possible_reasons, &mut 100)?;

            unsafe_fns_in_cur = possible_reasons.clone().into();

            if reasons.is_empty() {
                // take the first possible unsafe fn, if it is local, check why it is unsafe.
                'possible: while let Some(violation) = possible_reasons.pop_front() {
                    tracing::debug!(
                        "unsafe fn {:?} is a possible reason for unsafety",
                        violation
                    );
                    match violation.0 {
                        UnsafeOpKind::CallToUnsafeFunction(Some(new_did)) => {
                            if let Some(new_did) = new_did.as_local() {
                                did = new_did;
                            } else {
                                // A unlocal function is unsafe, so we can stop looking.
                                reasons.push(violation)
                            }
                            break 'possible;
                        }
                        _ => continue 'possible,
                    }
                }
            } else {
                break;
            }
        }
        reasons.extend(unsafe_fns_in_cur);
        Ok(reasons)
    }

    fn find_unsafe_things_<'tcx>(
        &self,
        tcx: ty::TyCtxt<'tcx>,
        def_id: LocalDefId,
        reasons: &mut Vec<(UnsafeOpKind, Span)>,
        possible_reasons: &mut VecDeque<(UnsafeOpKind, Span)>,
        depth: &mut i32,
    ) -> Result<(), color_eyre::Report> {
        tracing::debug!(
            "finding out why {} is unsafe",
            tcx.def_path_str(def_id.to_def_id())
        );
        *depth -= 1;
        if *depth == 0 {
            panic!()
        }
        let res = unsafety_visitor::check_unsafety(tcx, ty::WithOptConstParam::unknown(def_id));
        let mut unsafe_found = false;
        for violation in &res {
            match violation.0 {
                UnsafeOpKind::CallToUnsafeFunction(_) => {
                    possible_reasons.push_back(violation.clone());
                    unsafe_found = true;
                }
                _ => {
                    reasons.push(violation.clone());
                    unsafe_found = true;
                }
            }
        }

        if !unsafe_found {
            reasons.push((UnsafeOpKind::ChoosenUnsafe, tcx.def_span(def_id)))
        }
        Ok(())
    }

    /// Search for the selectors defid
    pub fn search<'tcx>(&self, tcx: ty::TyCtxt<'tcx>) -> Result<(LocalDefId, FnHeader)> {
        struct Searcher<'a, 't> {
            selector: &'a str,
            result: Option<(LocalDefId, FnHeader)>,
            tcx: TyCtxt<'t>,
        }

        impl Searcher<'_, '_> {
            fn check_match(&mut self, def_id: LocalDefId) -> bool {
                let path_str = self.tcx.def_path_str(def_id.to_def_id());
                tracing::debug!(?path_str);

                path_str.ends_with(&self.selector)
            }
        }

        impl<'hir, 'a, 't> ItemLikeVisitor<'hir> for Searcher<'a, 't> {
            fn visit_item(&mut self, item: &'hir hir::Item<'hir>) {
                if let hir::Item{ kind: hir::ItemKind::Fn( fn_sig, _, _), ..} = item && self.check_match(item.def_id) && self.result.is_none() {
                    self.result = Some((item.def_id, fn_sig.header));
                }
            }

            fn visit_impl_item(&mut self, item_impl: &'hir hir::ImplItem<'hir>) {
                if let hir::ImplItem{kind: hir::ImplItemKind::Fn(fn_sig, _), ..} = item_impl && self.check_match(item_impl.def_id) && self.result.is_none() {
                    self.result = Some((item_impl.def_id, fn_sig.header));
                }
            }

            fn visit_foreign_item(&mut self, _: &'hir hir::ForeignItem<'hir>) {}
            fn visit_trait_item(&mut self, _: &'hir hir::TraitItem<'hir>) {}
        }
        // Find def_id of function in selector.
        let hir = tcx.hir();
        let mut visitor = Searcher {
            selector: &self.selector,
            result: None,
            tcx,
        };
        hir.visit_all_item_likes(&mut visitor);
        visitor.result.ok_or(eyre::eyre!("no such function found"))
    }
}

impl rustc_driver::Callbacks for FakeCallback {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        config.override_queries = Some(|_, p, _| p.thir_check_unsafety = check_unsafety);
    }
}

pub fn check_unsafety<'tcx>(
    tcx: TyCtxt<'tcx>,
    _: rustc_middle::ty::query::query_keys::thir_check_unsafety<'tcx>,
) {
    let selector = std::env::var(crate::ENV_VAR_WHYNOT_SELECTOR)
        .wrap_err(crate::WHYNOT_RUSTC_WRAPPER_ERROR)
        .unwrap();
    FakeCallback { selector }.run(tcx).unwrap();
    std::process::exit(0);
}
