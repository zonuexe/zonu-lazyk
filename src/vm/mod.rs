//! The ION-style combinator VM (ADR-0001): a flat-array heap reduced with an
//! explicit spine stack, collected by a Cheney copying GC (ADR-0002).

pub mod gc;
pub mod heap;
pub mod reduce;

use crate::term::Term;
use heap::{Heap, Ref};

/// A loaded program ready to run.
pub struct Vm {
    pub heap: Heap,
    /// Root of the program graph.
    pub root: Ref,
    /// The spine stack — GC roots plus the reducer's working stack.
    pub spine: Vec<Ref>,
}

impl Vm {
    /// Load an optimized term into a fresh heap.
    pub fn load(term: &Term) -> Vm {
        let mut heap = Heap::new();
        let root = heap.alloc_term(term);
        Vm {
            heap,
            root,
            spine: Vec::new(),
        }
    }

    /// Run the program: `output = program input`, streaming bytes.
    pub fn run<R: std::io::Read, W: std::io::Write>(
        &mut self,
        input: R,
        output: W,
    ) -> Result<(), crate::Error> {
        crate::io::drive(self, input, output)
    }
}
