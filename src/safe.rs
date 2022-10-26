pub mod expr;
pub mod unsafety_visitor;

use std::{collections::VecDeque, ffi::OsString};

use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle};
use eyre::{Context, Result};
use hir::{def_id::LocalDefId, intravisit::Visitor, FnHeader};
use itertools::Itertools;
use rustc_hir as hir;
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;

use crate::{run::cargo_check, safe::unsafety_visitor::UnsafeOpKind};

pub(crate) fn run(args: crate::opts::Args, rem: &[String]) -> Result<()> {
    std::env::set_var(crate::ENV_VAR_WHYNOT_COLORING, args.color.to_string());
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
    tracing::trace!("in whynot safe rustc with selector: {selector:?}");
    tracing::trace!("in whynot safe rustc with rem: `{rem:?}`");

    crate::run::rustc_run(Some(&mut FakeCallback { selector }), None, &rem[1..])?;

    Ok(())
}

pub struct SafeOutput<'s> {
    /// This vec is ordered so that the most serious violations are first.
    reasons: Vec<(UnsafeOpKind, LocalDefId, Span)>,
    source_map: &'s rustc_span::source_map::SourceMap,
}

impl SafeOutput<'_> {
    //
    pub fn print(
        self,
        mut io: termcolor::StandardStream,
        tcx: TyCtxt<'_>,
        checked_fn: LocalDefId,
    ) -> Result<()> {
        let mut files = codespan_reporting::files::SimpleFiles::new();
        let mut hash_map = std::collections::HashMap::new();

        {
            let idx = self
                .source_map
                .lookup_source_file_idx(tcx.def_span(checked_fn).lo());
            let file = self.source_map.files()[idx].clone();
            if let Some(src) = file.src.clone() {
                let f_idx = files.add(file.name.prefer_local().to_string(), (*src).clone());
                hash_map.entry(idx).or_insert(f_idx);
            }
        }

        for (_, _, span) in self.reasons.iter() {
            let idx = self.source_map.lookup_source_file_idx(span.lo());
            if hash_map.contains_key(&idx) {
                continue;
            }
            let file = self.source_map.files()[idx].clone();
            if let Some(src) = file.src.clone() {
                let f_idx = files.add(file.name.prefer_local().to_string(), (*src).clone());
                hash_map.entry(idx).or_insert(f_idx);
            }
        }

        let mut labels = vec![];
        let mut first = true;
        // why is this unique needed?
        for (did, reasons) in &self
            .reasons
            .into_iter()
            .unique_by(|(_, _, span)| *span)
            .sorted_by_key(|(_, did, _)| did.local_def_index)
            .group_by(|(_, did, _)| *did)
        {
            let idx = self
                .source_map
                .lookup_source_file_idx(tcx.def_span(did).lo());
            labels.push(span_label(
                hash_map[&idx],
                self.source_map,
                tcx.def_span(did),
                LabelStyle::Primary,
                Some("function is unsafe because:".to_string()),
            ));
            let mut primary_reason = false;
            let mut primary_reason_is_extern = true;
            for (reason, _, span) in reasons {
                let idx = self.source_map.lookup_source_file_idx(span.lo());
                let label = match reason {
                    UnsafeOpKind::CallToUnsafeFunction(Some(did))
                    | UnsafeOpKind::CallToFunctionWith(did)
                        if did.as_local().is_some() =>
                    {
                        span_label(
                            hash_map[&idx],
                            self.source_map,
                            span,
                            LabelStyle::Secondary,
                            Some(reason.description_and_note(tcx).0.to_string()),
                        )
                    }
                    _ => {
                        primary_reason = true;
                        if let UnsafeOpKind::CallToUnsafeFunction(Some(_))
                        | UnsafeOpKind::CallToFunctionWith(_) = reason
                        {
                        } else {
                            primary_reason_is_extern = false;
                        }
                        span_label(
                            hash_map[&idx],
                            self.source_map,
                            span,
                            LabelStyle::Primary,
                            Some(reason.simple_description().to_string()),
                        )
                    }
                };
                labels.push(label);
            }

            let mut diag = if first {
                first = false;
                Diagnostic::note()
                    .with_message("Function is unsafe")
                    .with_labels(labels.clone())
            } else {
                Diagnostic::help().with_labels(labels.clone())
            };

            if primary_reason {
                if primary_reason_is_extern {
                    diag = diag.with_notes(vec![
                        "this function calls an external unsafe function".to_string(),
                    ]);
                } else {
                    diag = diag.with_notes(vec![
                        "this function does a fundamentally unsafe operation".to_string(),
                    ]);
                }
            }
            codespan_reporting::term::emit(
                &mut io,
                &codespan_reporting::term::Config {
                    display_style: codespan_reporting::term::DisplayStyle::Rich,
                    //tab_width: todo!(),
                    //styles: todo!(),
                    //chars: todo!(),
                    //start_context_lines: todo!(),
                    //end_context_lines: todo!(),
                    ..<_>::default()
                },
                &files,
                &diag,
            )?;
            labels.clear();
        }

        Ok(())
    }
}

pub fn span_label<FileId>(
    id: FileId,
    sm: &rustc_span::source_map::SourceMap,
    span: Span,
    style: LabelStyle,
    message: Option<String>,
) -> Label<FileId> {
    let start = sm.lookup_byte_offset(span.lo()).pos.0 as usize;
    let end = sm.lookup_byte_offset(span.hi()).pos.0 as usize;
    let l = Label::new(style, id, std::ops::Range { start, end });
    if let Some(msg) = message {
        l.with_message(msg)
    } else {
        l
    }
}
pub struct FakeCallback {
    selector: String,
}

impl FakeCallback {
    pub fn run(&self, tcx: ty::TyCtxt<'_>) -> Result<()> {
        let (fun_id, header) = self.search(tcx)?;
        tracing::trace!(?header);
        if header.unsafety == hir::Unsafety::Normal {
            println!("function is not unsafe");
            return Ok(());
        }
        let reasons = self.find_unsafe_things(tcx, fun_id)?;

        let color: crate::opts::Coloring = std::env::var(crate::ENV_VAR_WHYNOT_COLORING)
            .wrap_err(crate::WHYNOT_RUSTC_WRAPPER_ERROR)?
            .parse()
            .wrap_err(crate::WHYNOT_RUSTC_WRAPPER_ERROR)?;
        SafeOutput {
            reasons,
            source_map: tcx.sess.source_map(),
        }
        .print(termcolor::StandardStream::stdout(color.into()), tcx, fun_id)?;
        Ok(())
    }

    pub fn find_unsafe_things(
        &self,
        tcx: ty::TyCtxt<'_>,
        fun_id: LocalDefId,
    ) -> Result<Vec<(UnsafeOpKind, LocalDefId, Span)>> {
        let mut reasons = vec![];
        let mut possible_reasons: VecDeque<_> = vec![].into();
        let mut did = fun_id;
        let mut loopctr = 10;
        loop {
            loopctr -= 1;
            if loopctr == 0 {
                break;
            }

            self.find_unsafe_things_(tcx, did, &mut reasons, &mut possible_reasons, &mut 100)?;
            tracing::debug!(reasons = ?reasons, possible_reasons = ?possible_reasons, "found unsafe things");
            if reasons.is_empty() {
                // take the first possible unsafe fn, if it is local, check why it is unsafe.
                'possible: while let Some(violation) = possible_reasons.pop_front() {
                    // tracing::debug!(
                    //     "unsafe fn {:?} is a possible reason for unsafety",
                    //     violation
                    // );
                    reasons.push(violation);

                    match violation.0 {
                        UnsafeOpKind::CallToUnsafeFunction(Some(new_did)) => {
                            if let Some(new_did) = new_did.as_local() {
                                did = new_did;
                            } else {
                                // A unlocal function is unsafe, so we can stop looking.
                            }
                            break 'possible;
                        }
                        _ => continue 'possible,
                    }
                }
            } else {
                if possible_reasons.is_empty() {
                    break;
                }
                reasons.extend(possible_reasons.iter().cloned());
            }
        }
        Ok(reasons)
    }

    fn find_unsafe_things_<'tcx>(
        &self,
        tcx: ty::TyCtxt<'tcx>,
        def_id: LocalDefId,
        reasons: &mut Vec<(UnsafeOpKind, LocalDefId, Span)>,
        possible_reasons: &mut VecDeque<(UnsafeOpKind, LocalDefId, Span)>,
        depth: &mut i32,
    ) -> Result<(), color_eyre::Report> {
        tracing::trace!(
            "finding out why {} is unsafe",
            tcx.def_path_str(def_id.to_def_id())
        );
        *depth -= 1;
        if *depth == 0 {
            panic!()
        }
        let res = unsafety_visitor::check_unsafety(tcx, ty::WithOptConstParam::unknown(def_id));
        tracing::debug!("res: {res:?}");
        let mut unsafe_found = false;
        for violation in &res {
            match violation.0 {
                UnsafeOpKind::CallToUnsafeFunction(_) => {
                    possible_reasons.push_back(*violation);
                    unsafe_found = true;
                }
                _ => {
                    reasons.push(*violation);
                    unsafe_found = true;
                }
            }
        }

        if !unsafe_found {
            reasons.push((UnsafeOpKind::ChoosenUnsafe, def_id, tcx.def_span(def_id)))
        }
        Ok(())
    }

    /// Search for the selectors defid
    pub fn search(&self, tcx: ty::TyCtxt<'_>) -> Result<(LocalDefId, FnHeader)> {
        struct Searcher<'a, 't> {
            selector: &'a str,
            result: Option<(LocalDefId, FnHeader)>,
            tcx: TyCtxt<'t>,
        }

        impl Searcher<'_, '_> {
            fn check_match(&mut self, def_id: LocalDefId) -> bool {
                let path_str = self.tcx.def_path_str(def_id.to_def_id());
                tracing::trace!(?path_str);

                path_str.ends_with(&self.selector)
            }
        }

        impl<'hir, 'a, 't> Visitor<'hir> for Searcher<'a, 't> {
            fn visit_item(&mut self, item: &'hir hir::Item<'hir>) {
                if let hir::Item{ kind: hir::ItemKind::Fn( fn_sig, _, _), ..} = item && self.check_match(item.def_id.def_id) && self.result.is_none() {
                    self.result = Some((item.def_id.def_id, fn_sig.header));
                }
            }

            fn visit_impl_item(&mut self, item_impl: &'hir hir::ImplItem<'hir>) {
                if let hir::ImplItem{kind: hir::ImplItemKind::Fn(fn_sig, _), ..} = item_impl && self.check_match(item_impl.def_id.def_id) && self.result.is_none() {
                    self.result = Some((item_impl.def_id.def_id, fn_sig.header));
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
        hir.visit_all_item_likes_in_crate(&mut visitor);
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
