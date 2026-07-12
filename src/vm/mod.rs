//! The ION-style combinator VM (ADR-0001): a flat-array heap reduced with an
//! explicit spine stack, collected by a Cheney copying GC (ADR-0002).

pub mod gc;
pub mod heap;
pub mod reduce;

use crate::term::Term;
use heap::{Cell, Heap, Ref};
use std::io::BufRead;

/// Heap size (in cells) below which the collector never runs.
const GC_FLOOR: usize = 1 << 16;

/// A loaded program together with its runtime heap and reduction state.
pub struct Vm {
    pub heap: Heap,
    /// Root of the program graph (the combinator term to apply to the input).
    pub root: Ref,
    /// Shared reduction stack. Nested `whnf` calls stack their spines on top of
    /// the caller's, remembering a base index; the whole vector is a GC root.
    pub(crate) spine: Vec<Ref>,
    /// Extra GC roots that live across `whnf` calls (the I/O driver's refs).
    pub(crate) roots: Vec<Ref>,
    /// Collect once the heap grows to this many cells.
    pub(crate) gc_threshold: usize,
    /// Reduction-step ceiling (`u64::MAX` = unlimited); the count so far; and
    /// whether the ceiling was hit (ADR-0007). Checked once per `whnf` iteration.
    pub(crate) max_steps: u64,
    pub(crate) steps: u64,
    pub(crate) step_limit_hit: bool,
    /// Output-byte ceiling (`u64::MAX` = unlimited); enforced by the I/O driver.
    pub(crate) max_output: u64,
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
            spine: Vec::new(),
            roots: Vec::new(),
            gc_threshold: GC_FLOOR,
            max_steps: u64::MAX,
            steps: 0,
            step_limit_hit: false,
            max_output: u64::MAX,
            input: None,
        }
    }

    /// Override the collection threshold (used by tests to force frequent GC).
    #[doc(hidden)]
    pub fn set_gc_threshold(&mut self, cells: usize) {
        self.gc_threshold = cells.max(64);
    }

    /// Apply opt-in resource limits (ADR-0007). `None` means unlimited.
    pub(crate) fn set_limits(&mut self, limits: &crate::Limits) {
        self.max_steps = limits.max_steps.unwrap_or(u64::MAX);
        self.max_output = limits.max_output_bytes.unwrap_or(u64::MAX);
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

    /// Run over `input` and collect the output list as raw Church numerals
    /// (ADR-0008). `eof` is the end sentinel; `max_values` a soft element cap.
    pub fn run_values<R: std::io::Read + 'static>(
        &mut self,
        input: R,
        eof: Option<u64>,
        max_values: Option<u64>,
    ) -> Result<Vec<u64>, crate::Error> {
        self.input = Some(Box::new(std::io::BufReader::new(input)));
        let res = crate::io::collect_values(self, eof, max_values);
        self.input = None;
        res
    }

    /// Evaluate the program term itself as a Church numeral (ADR-0008).
    pub fn eval_numeral(&mut self) -> Result<u64, crate::Error> {
        crate::io::eval_numeral(self)
    }

    /// Read the next input byte, or the EOF numeral 256 at end of stream. Fits a
    /// `u32` (the input `Num` cells are bytes `0..=256`).
    pub(crate) fn read_input_byte(&mut self) -> u32 {
        const INPUT_EOF: u32 = 256;
        let mut buf = [0u8; 1];
        match self.input.as_mut() {
            Some(r) => match r.read(&mut buf) {
                Ok(1) => buf[0] as u32,
                _ => INPUT_EOF,
            },
            None => INPUT_EOF,
        }
    }

    /// Collect the heap, remapping the spine and root vectors.
    pub(crate) fn gc(&mut self) {
        {
            let Vm {
                heap, spine, roots, ..
            } = self;
            gc::collect(heap.cells_mut(), spine, roots);
        }
        // Grow the threshold to keep amortized collection cost low.
        self.gc_threshold = GC_FLOOR.max(self.heap.len() * 2);
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
