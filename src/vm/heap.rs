//! Flat-array cell heap (ADR-0002).
//!
//! Cells are addressed by `Ref` (a `u32` index), never by pointer, so the
//! Cheney collector can relocate them and rewrite indices. Sharing is realised
//! by overwriting a redex root with an `Ind` (indirection) cell.

use crate::term::{Comb, Term};

/// Index of a cell in the heap.
pub type Ref = u32;

/// A single heap cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    /// Application `a b` — operator `a` applied to operand `b`.
    App(Ref, Ref),
    /// A combinator symbol.
    Comb(Comb),
    /// A native integer Church numeral (ADR-0004).
    Num(u32),
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
    cells: Vec<Cell>,
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
        // TODO(ADR-0002): trigger `gc::collect` when a fill threshold is reached.
        let r = self.cells.len() as Ref;
        self.cells.push(cell);
        r
    }

    #[inline]
    pub fn get(&self, r: Ref) -> Cell {
        self.cells[r as usize]
    }

    #[inline]
    pub fn set(&mut self, r: Ref, cell: Cell) {
        self.cells[r as usize] = cell;
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

    /// Mutable access to the backing cell vector, for the collector.
    pub(crate) fn cells_mut(&mut self) -> &mut Vec<Cell> {
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
