use rustc::hir::*;
use rustc::lint::*;

use crate::utils;

const ORD: [&str; 3] = ["core", "cmp", "Ord"];
const PARTIAL_ORD: [&str; 3] = ["core", "cmp", "PartialOrd"];

/// **What it does:**
/// Checks for the usage of negated comparision operators on types which only implement
/// `PartialOrd` (e.g. `f64`).
///
/// **Why is this bad?**
/// These operators make it easy to forget that the underlying types actually allow not only three
/// potential Orderings (Less, Equal, Greater) but also a forth one (Uncomparable). Escpeccially if
/// the operator based comparision result is negated it is easy to miss that fact.
///
/// **Known problems:** None.
///
/// **Example:**
///
/// ```rust
/// use core::cmp::Ordering;
/// 
/// // Bad
/// let a = 1.0;
/// let b = std::f64::NAN;
/// 
/// let _not_less_or_equal = !(a <= b);
///
/// // Good
/// let a = 1.0;
/// let b = std::f64::NAN;
/// 
/// let _not_less_or_equal = match a.partial_cmp(&b) {
///     None | Some(Ordering::Greater) => true,
///     _ => false, 
/// };
/// ```
declare_clippy_lint! {
    pub NEG_CMP_OP_ON_PARTIAL_ORD,
    complexity,
    "The use of negated comparision operators on partially orded types may produce confusing code."
}

pub struct NoNegCompOpForPartialOrd;

impl LintPass for NoNegCompOpForPartialOrd {
    fn get_lints(&self) -> LintArray {
        lint_array!(NEG_CMP_OP_ON_PARTIAL_ORD)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for NoNegCompOpForPartialOrd {

    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        if_chain! {

            if let Expr_::ExprUnary(UnOp::UnNot, ref inner) = expr.node;
            if let Expr_::ExprBinary(ref op, ref left, _) = inner.node;
            if let BinOp_::BiLe | BinOp_::BiGe | BinOp_::BiLt | BinOp_::BiGt = op.node;

            then {

                let ty = cx.tables.expr_ty(left);

                let implements_ord = {
                    if let Some(id) = utils::get_trait_def_id(cx, &ORD) {
                        utils::implements_trait(cx, ty, id, &[])
                    } else {
                        return;
                    }
                };

                let implements_partial_ord = {
                    if let Some(id) = utils::get_trait_def_id(cx, &PARTIAL_ORD) {
                        utils::implements_trait(cx, ty, id, &[])
                    } else {
                        return;
                    }
                };

                if implements_partial_ord && !implements_ord {
                    cx.span_lint(
                        NEG_CMP_OP_ON_PARTIAL_ORD,
                        expr.span,
                        "The use of negated comparision operators on partially orded\
                        types produces code that is hard to read and refactor. Please\
                        consider to use the partial_cmp() instead, to make it clear\
                        that the two values could be incomparable."
                    )
                }
            }
        }
    }
}
