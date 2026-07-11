//! Compile-time peephole optimizer (ADR-0003).
//!
//! Rewrites local SKI patterns into extended combinators to cut reduction
//! steps, preserving observable behaviour. Applied bottom-up so that inner
//! rewrites expose outer ones.
//!
//! Level 1 (introduce B / C / collapse K):
//!   S (K p) (K q)  ->  K (p q)
//!   S (K p) I      ->  p
//!   S (K p) q      ->  B p q
//!   S p     (K q)  ->  C p q
//!
//! Level 2 (balanced combinators, fire when the left arg is `B k f`):
//!   S (B k f) g    ->  S' k f g
//!   B (B k f) g    ->  B' k f g
//!   C (B k f) g    ->  C' k f g
//!
//! Every rule is an extensional equality; the reducer implements the matching
//! rewrite rules, and differential tests pin optimized == unoptimized output.

use crate::term::{Comb, Term};
use std::rc::Rc;

/// Apply the peephole rewrites bottom-up.
pub fn optimize(term: Term) -> Term {
    match term {
        Term::App(f, x) => {
            let f = optimize(Rc::unwrap_or_clone(f));
            let x = optimize(Rc::unwrap_or_clone(x));
            apply(f, x)
        }
        other => other,
    }
}

fn comb(c: Comb) -> Term {
    Term::comb(c)
}

fn app(f: Term, x: Term) -> Term {
    Term::app(f, x)
}

fn is_comb(t: &Term, c: Comb) -> bool {
    matches!(t, Term::Comb(k) if *k == c)
}

/// If `t == App(head, arg)`, return `(head, arg)`.
fn split(t: &Term) -> Option<(&Term, &Term)> {
    match t {
        Term::App(f, x) => Some((f, x)),
        _ => None,
    }
}

/// Return `(k, f)` if `t == B k f` i.e. `App(App(B, k), f)`.
fn as_b_app(t: &Term) -> Option<(&Term, &Term)> {
    let (bk, f) = split(t)?;
    let (b, k) = split(bk)?;
    if is_comb(b, Comb::B) {
        Some((k, f))
    } else {
        None
    }
}

/// Build `f x`, applying the peephole rewrites (`f` and `x` already optimized).
fn apply(f: Term, x: Term) -> Term {
    if let Some((op, a)) = split(&f) {
        // ---- head is `S a`, so this node is `S a x` ----
        if is_comb(op, Comb::S) {
            if let Some((ka, p)) = split(a)
                && is_comb(ka, Comb::K)
            {
                // S (K p) x
                if let Some((kb, q)) = split(&x)
                    && is_comb(kb, Comb::K)
                {
                    return app(comb(Comb::K), app(p.clone(), q.clone())); // K (p q)
                }
                if is_comb(&x, Comb::I) {
                    return p.clone(); // S (K p) I = p
                }
                return app(app(comb(Comb::B), p.clone()), x); // B p x
            }
            if let Some((k, inner_f)) = as_b_app(a) {
                // S (B k f) x = S' k f x
                return app(app(app(comb(Comb::Sp), k.clone()), inner_f.clone()), x);
            }
            if let Some((kb, q)) = split(&x)
                && is_comb(kb, Comb::K)
            {
                return app(app(comb(Comb::C), a.clone()), q.clone()); // C a q
            }
        }
        // ---- head is `B a`, so this node is `B a x` ----
        if is_comb(op, Comb::B)
            && let Some((k, inner_f)) = as_b_app(a)
        {
            // B (B k f) x = B' k f x
            return app(app(app(comb(Comb::Bp), k.clone()), inner_f.clone()), x);
        }
        // ---- head is `C a`, so this node is `C a x` ----
        if is_comb(op, Comb::C)
            && let Some((k, inner_f)) = as_b_app(a)
        {
            // C (B k f) x = C' k f x
            return app(app(app(comb(Comb::Cp), k.clone()), inner_f.clone()), x);
        }
    }
    app(f, x)
}

#[cfg(test)]
mod tests {
    use super::optimize;
    use crate::term::{Comb, Term};

    fn c(k: Comb) -> Term {
        Term::comb(k)
    }
    fn ap(f: Term, x: Term) -> Term {
        Term::app(f, x)
    }

    #[test]
    fn s_kp_kq_becomes_k_pq() {
        use Comb::*;
        // S (K K) (K I)  ->  K (K I)
        let t = ap(ap(c(S), ap(c(K), c(K))), ap(c(K), c(I)));
        assert_eq!(optimize(t), ap(c(K), ap(c(K), c(I))));
    }

    #[test]
    fn s_kp_i_becomes_p() {
        use Comb::*;
        // S (K K) I  ->  K
        let t = ap(ap(c(S), ap(c(K), c(K))), c(I));
        assert_eq!(optimize(t), c(K));
    }

    #[test]
    fn s_kp_q_becomes_b() {
        use Comb::*;
        // S (K I) (S K K)  ->  B I (S K K)
        let skk = ap(ap(c(S), c(K)), c(K));
        let t = ap(ap(c(S), ap(c(K), c(I))), skk.clone());
        assert_eq!(optimize(t), ap(ap(c(B), c(I)), skk));
    }

    #[test]
    fn s_p_kq_becomes_c() {
        use Comb::*;
        // S K (K I)  ->  C K I
        let t = ap(ap(c(S), c(K)), ap(c(K), c(I)));
        assert_eq!(optimize(t), ap(ap(c(C), c(K)), c(I)));
    }

    #[test]
    fn s_bkf_g_becomes_sp() {
        use Comb::*;
        // S (B K I) I  ->  S' K I I
        let t = ap(ap(c(S), ap(ap(c(B), c(K)), c(I))), c(I));
        assert_eq!(optimize(t), ap(ap(ap(c(Sp), c(K)), c(I)), c(I)));
    }

    #[test]
    fn b_bkf_g_becomes_bp() {
        use Comb::*;
        // B (B I I) I  ->  B' I I I
        let t = ap(ap(c(B), ap(ap(c(B), c(I)), c(I))), c(I));
        assert_eq!(optimize(t), ap(ap(ap(c(Bp), c(I)), c(I)), c(I)));
    }

    #[test]
    fn c_bkf_g_becomes_cp() {
        use Comb::*;
        // C (B K I) I  ->  C' K I I
        let t = ap(ap(c(C), ap(ap(c(B), c(K)), c(I))), c(I));
        assert_eq!(optimize(t), ap(ap(ap(c(Cp), c(K)), c(I)), c(I)));
    }

    #[test]
    fn plain_ski_is_unchanged() {
        use Comb::*;
        let skk = ap(ap(c(S), c(K)), c(K));
        assert_eq!(optimize(skk.clone()), skk);
    }
}
