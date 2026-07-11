//! Compile-time peephole optimizer (ADR-0003).
//!
//! Rewrites local SKI patterns into extended combinators to cut reduction
//! steps, preserving observable behaviour:
//!
//!   S (K p) (K q)  ->  K (p q)
//!   S (K p) I      ->  p
//!   S (K p) q      ->  B p q
//!   S p (K q)      ->  C p q
//!   ... (and the B'/C'/S' families for nested spines)
//!
//! Each rule is an extensional equality; a wrong rule silently changes results,
//! so every rule carries reference-program test coverage.

use crate::term::Term;

/// Apply the peephole rewrites to a fixpoint (bottom-up).
pub fn optimize(term: Term) -> Term {
    // TODO: recurse bottom-up, rewriting `App` nodes via the rule table above,
    // re-running locally until no rule fires.
    term
}
