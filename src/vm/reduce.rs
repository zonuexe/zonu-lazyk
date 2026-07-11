//! Normal-order reduction to WHNF via an explicit spine stack (ADR-0001).
//!
//! Unwind the leftmost spine onto `Vm::spine`, and when the head is a combinator
//! with `arity()` arguments available, fire its rewrite rule and overwrite the
//! redex root with an `Ind` to the result (sharing).

use super::Vm;
use super::heap::Ref;

impl Vm {
    /// Reduce the cell at `root` to weak head normal form and return its (possibly
    /// relocated) reference.
    pub fn whnf(&mut self, root: Ref) -> Ref {
        // TODO: the unwind/rewrite loop:
        //   loop {
        //     unwind App spine onto self.spine
        //     match head {
        //       Comb(c) if args >= c.arity() => apply_rule(c); overwrite root with Ind
        //       Comb(_) | Num(_)             => break, // WHNF
        //       Ind(t)                       => follow
        //     }
        //   }
        let _ = root;
        todo!("reduce::whnf")
    }
}
