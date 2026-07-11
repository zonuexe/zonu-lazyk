//! Flat-array cell heap (ADR-0002).
//!
//! Cells are addressed by `Ref` (a `u32` index), never by pointer, so the
//! Cheney collector can relocate them and rewrite indices. Sharing is realised
//! by overwriting a redex root with an `Ind` (indirection) cell.

use crate::term::{Comb, Term};

/// Index of a cell in the heap.
pub type Ref = u32;

/// A single heap cell. Fixed size, uniform layout so GC can copy blindly.
#[derive(Debug, Clone, Copy)]
pub enum Cell {
    /// Application `a b` — operator `a` applied to operand `b`.
    App(Ref, Ref),
    /// A combinator symbol.
    Comb(Comb),
    /// A native integer Church numeral (ADR-0004).
    Num(u32),
    /// Indirection to another cell (result of an in-place update).
    Ind(Ref),
}

pub struct Heap {
    cells: Vec<Cell>,
}

impl Heap {
    pub fn new() -> Heap {
        Heap { cells: Vec::new() }
    }

    pub fn alloc(&mut self, cell: Cell) -> Ref {
        // TODO: bump-allocate into to-space; trigger `gc::collect` when full.
        let r = self.cells.len() as Ref;
        self.cells.push(cell);
        r
    }

    pub fn get(&self, r: Ref) -> Cell {
        self.cells[r as usize]
    }

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
