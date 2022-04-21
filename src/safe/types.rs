use rustc_hash::FxHashMap;
use rustc_macros::TyEncodable;
use rustc_macros::HashStable;
use rustc_macros::TyDecodable;
use rustc_middle::{
    mir::{
        SourceInfo, UnsafetyViolationDetails, UnsafetyViolationKind, UnusedUnsafe,
        UsedUnsafeBlockData, self,
    }, ty::codec::{TyEncoder, RefDecodable, TyDecoder},
};
use rustc_hir as hir;
use rustc_serialize::{Encodable, Decodable};

#[derive(TyEncodable, TyDecodable, HashStable, Debug)]
pub struct UnsafetyCheckResult {
    /// Violations that are propagated *upwards* from this function.
    pub violations: Vec<UnsafetyViolation>,

    /// Used `unsafe` blocks in this function. This is used for the "unused_unsafe" lint.
    ///
    /// The keys are the used `unsafe` blocks, the UnusedUnsafeKind indicates whether
    /// or not any of the usages happen at a place that doesn't allow `unsafe_op_in_unsafe_fn`.
    pub used_unsafe_blocks: FxHashMap<hir::HirId, UsedUnsafeBlockData>,

    /// This is `Some` iff the item is not a closure.
    pub unused_unsafes: Option<Vec<(hir::HirId, UnusedUnsafe)>>,
}

impl<'tcx, D: TyDecoder<'tcx>> Decodable<D> for &'tcx UnsafetyCheckResult {
    fn decode(decoder: &mut D) -> Self {
        RefDecodable::decode(decoder)
    }
}

impl<'tcx, E: TyEncoder<'tcx>> Encodable<E> for &'tcx UnsafetyCheckResult {
    fn encode(&self, e: &mut E) -> Result<(), E::Error> {
        (**self).encode(e)
    }
}

#[derive(Copy, Clone, PartialEq, TyEncodable, TyDecodable, HashStable, Debug)]
pub struct UnsafetyViolation {
    pub source_info: SourceInfo,
    pub lint_root: hir::HirId,
    pub kind: UnsafetyViolationKind,
    pub details: UnsafetyViolationDetails,
}
