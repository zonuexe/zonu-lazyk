//! The core combinator term produced by the parser and rewritten by the compiler.
//!
//! Input is pure SKI; the peephole pass (ADR-0003) introduces the extended
//! combinators. `Num` exists so the I/O boundary can inject native integers
//! (ADR-0004); it never appears in freshly parsed programs.

use std::rc::Rc;

/// A combinator symbol. `S`/`K`/`I` come from the source; the rest are
/// introduced by the peephole optimizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comb {
    S,
    K,
    I,
    /// `B f g x = f (g x)`
    B,
    /// `C f g x = f x g`
    C,
    /// `S' c f g x = c (f x) (g x)`
    Sp,
    /// `B' c f g x = c f (g x)`
    Bp,
    /// `C' c f g x = c (f x) g`
    Cp,
    /// Successor primitive over native `Num` (I/O boundary).
    Inc,
}

impl Comb {
    /// Arity — how many spine arguments the rewrite rule consumes.
    pub const fn arity(self) -> usize {
        match self {
            Comb::I | Comb::Inc => 1,
            Comb::K => 2,
            Comb::S | Comb::B | Comb::C => 3,
            Comb::Sp | Comb::Bp | Comb::Cp => 4,
        }
    }
}

/// A parsed / optimized term. Reference-counted so the peephole pass can share
/// subterms cheaply before they are loaded into the VM heap.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Comb(Comb),
    Num(u32),
    App(Rc<Term>, Rc<Term>),
}

impl Term {
    pub fn comb(c: Comb) -> Term {
        Term::Comb(c)
    }

    pub fn app(f: Term, x: Term) -> Term {
        Term::App(Rc::new(f), Rc::new(x))
    }
}
