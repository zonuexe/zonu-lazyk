//! The ION-style combinator VM (ADR-0001): a flat-array heap reduced with an
//! explicit spine stack, collected by a Cheney copying GC (ADR-0002, TODO).

pub mod gc;
pub mod heap;
pub mod reduce;

use crate::term::Term;
use heap::{Cell, Heap, Ref};
use std::io::BufRead;

/// A loaded program together with its runtime heap.
pub struct Vm {
    pub heap: Heap,
    /// Root of the program graph (the combinator term to apply to the input).
    pub root: Ref,
    /// The input byte source, installed for the duration of [`Vm::run`].
    pub(crate) input: Option<Box<dyn BufRead>>,
}

impl Vm {
    /// Load an optimized term into a fresh heap.
    pub fn load(term: &Term) -> Vm {
        let mut heap = Heap::new();
        let root = heap.alloc_term(term);
        Vm {
            heap,
            root,
            input: None,
        }
    }

    /// Run the program: `output = program input`, streaming bytes.
    pub fn run<R: std::io::Read + 'static, W: std::io::Write>(
        &mut self,
        input: R,
        output: W,
    ) -> Result<(), crate::Error> {
        self.input = Some(Box::new(std::io::BufReader::new(input)));
        let res = crate::io::drive(self, output);
        self.input = None;
        res
    }

    /// Read the next input byte, or [`crate::io::EOF`] (256) at end of stream.
    pub(crate) fn read_input_byte(&mut self) -> u32 {
        let mut buf = [0u8; 1];
        match self.input.as_mut() {
            Some(r) => match r.read(&mut buf) {
                Ok(1) => buf[0] as u32,
                _ => crate::io::EOF,
            },
            None => crate::io::EOF,
        }
    }

    #[inline]
    pub(crate) fn alloc(&mut self, cell: Cell) -> Ref {
        self.heap.alloc(cell)
    }

    #[inline]
    pub(crate) fn app(&mut self, f: Ref, x: Ref) -> Ref {
        self.heap.alloc(Cell::App(f, x))
    }
}
