//! Flat-array cell heap (ADR-0002, ADR-0005).
//!
//! Cells are addressed by `Ref` (a `u32` index), never by pointer, so the
//! Cheney collector can relocate them and rewrite indices. Sharing is realised
//! by overwriting a redex root with an `Ind` (indirection) cell.
//!
//! [`Cell`] is the logical view; storage is a packed `u64` per cell (8 bytes vs
//! 12 for the enum) for cache density. Layout: tag in the top 3 bits; `App`/
//! `Cons` pack two 30-bit refs, the single-field variants use the low 32 bits.
//! Refs are therefore limited to 2^30 cells (an 8 GiB heap).

use crate::term::{Comb, Term};

/// Index of a cell in the heap.
pub type Ref = u32;

const TAG_SHIFT: u32 = 61;
const REF_MASK: u64 = (1 << 30) - 1;
const TAG_APP: u64 = 0;
const TAG_CONS: u64 = 1;
const TAG_IND: u64 = 2;
const TAG_NUM: u64 = 3;
const TAG_ACC: u64 = 4;
const TAG_COMB: u64 = 5;
const TAG_INPUT: u64 = 6;

#[inline]
fn comb_id(c: Comb) -> u64 {
    match c {
        Comb::S => 0,
        Comb::K => 1,
        Comb::I => 2,
        Comb::B => 3,
        Comb::C => 4,
        Comb::Sp => 5,
        Comb::Bp => 6,
        Comb::Cp => 7,
        Comb::Inc => 8,
    }
}

#[inline]
fn comb_from(id: u64) -> Comb {
    match id {
        0 => Comb::S,
        1 => Comb::K,
        2 => Comb::I,
        3 => Comb::B,
        4 => Comb::C,
        5 => Comb::Sp,
        6 => Comb::Bp,
        7 => Comb::Cp,
        _ => Comb::Inc,
    }
}

#[inline]
pub(crate) fn encode(cell: Cell) -> u64 {
    match cell {
        Cell::App(a, b) => (TAG_APP << TAG_SHIFT) | (a as u64) | ((b as u64) << 30),
        Cell::Cons(h, t) => (TAG_CONS << TAG_SHIFT) | (h as u64) | ((t as u64) << 30),
        Cell::Ind(t) => (TAG_IND << TAG_SHIFT) | (t as u64),
        Cell::Num(n) => (TAG_NUM << TAG_SHIFT) | (n as u64),
        Cell::Acc(k) => {
            debug_assert!(k < (1 << TAG_SHIFT), "Acc value exceeds 61 bits");
            (TAG_ACC << TAG_SHIFT) | k
        }
        Cell::Comb(c) => (TAG_COMB << TAG_SHIFT) | comb_id(c),
        Cell::Input => TAG_INPUT << TAG_SHIFT,
    }
}

#[inline]
pub(crate) fn decode(word: u64) -> Cell {
    match word >> TAG_SHIFT {
        TAG_APP => Cell::App((word & REF_MASK) as u32, ((word >> 30) & REF_MASK) as u32),
        TAG_CONS => Cell::Cons((word & REF_MASK) as u32, ((word >> 30) & REF_MASK) as u32),
        TAG_IND => Cell::Ind((word & REF_MASK) as u32),
        TAG_NUM => Cell::Num(word as u32),
        TAG_ACC => Cell::Acc(word & ((1 << TAG_SHIFT) - 1)),
        TAG_COMB => Cell::Comb(comb_from(word & 0xFF)),
        _ => Cell::Input,
    }
}

/// A single heap cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    /// Application `a b` — operator `a` applied to operand `b`.
    App(Ref, Ref),
    /// A combinator symbol.
    Comb(Comb),
    /// A native integer Church numeral (ADR-0004). Behaves as a Church numeral
    /// when applied: `Num(n) f x = f^n x`.
    Num(u32),
    /// The church2int counting accumulator (ADR-0004). Distinct from `Num` so
    /// it never collides with a program's Church numerals. `Acc(k) Inc = Acc(k+1)`.
    /// Holds up to 61 bits so a decoded numeral is not capped at 256 (ADR-0008).
    Acc(u64),
    /// Indirection to another cell (result of an in-place update).
    Ind(Ref),
    /// A native list cell that behaves as `\f. f head tail`. Used for the input
    /// stream; applying it to one argument `f` yields `f head tail`.
    Cons(Ref, Ref),
    /// A not-yet-read position in the input byte stream. Forcing it reads the
    /// next byte from stdin and rewrites this cell into a `Cons` (memoization),
    /// so each byte is read exactly once and in order.
    Input,
}

pub struct Heap {
    /// Packed cells — see the module docs for the `u64` layout.
    cells: Vec<u64>,
}

impl Heap {
    pub fn new() -> Heap {
        Heap { cells: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Heap {
        Heap {
            cells: Vec::with_capacity(cap),
        }
    }

    #[inline]
    pub fn alloc(&mut self, cell: Cell) -> Ref {
        let r = self.cells.len() as Ref;
        debug_assert!((r as u64) <= REF_MASK, "heap exceeded 2^30 cells");
        self.cells.push(encode(cell));
        r
    }

    #[inline]
    pub fn get(&self, r: Ref) -> Cell {
        decode(self.cells[r as usize])
    }

    #[inline]
    pub fn set(&mut self, r: Ref, cell: Cell) {
        self.cells[r as usize] = encode(cell);
    }

    /// Recursively materialize a [`Term`] into heap cells; returns its root.
    pub fn alloc_term(&mut self, term: &Term) -> Ref {
        match term {
            Term::Comb(c) => self.alloc(Cell::Comb(*c)),
            Term::Num(n) => self.alloc(Cell::Num(*n)),
            Term::App(f, x) => {
                let f = self.alloc_term(f);
                let x = self.alloc_term(x);
                self.alloc(Cell::App(f, x))
            }
        }
    }

    /// Mutable access to the backing packed-cell vector, for the collector.
    pub(crate) fn cells_mut(&mut self) -> &mut Vec<u64> {
        &mut self.cells
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl Default for Heap {
    fn default() -> Self {
        Heap::new()
    }
}
