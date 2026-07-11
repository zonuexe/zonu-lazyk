//! Cheney two-space copying collector (ADR-0002).
//!
//! Roots are the shared spine stack plus the explicit root vector (the I/O
//! driver's live refs). Live cells are copied breadth-first into a fresh space
//! and compacted; `Ind` chains are shortcut during copying, so the compacted
//! heap contains no indirections.

use super::heap::{Cell, Ref};

/// Resolve indirections from `r`, then copy the target into `to` if it has not
/// been copied yet, recording the forwarding index. Returns the new index.
fn forward(from: &[Cell], to: &mut Vec<Cell>, fwd: &mut [u32], mut r: Ref) -> Ref {
    while let Cell::Ind(t) = from[r as usize] {
        r = t;
    }
    let ru = r as usize;
    if fwd[ru] != u32::MAX {
        return fwd[ru];
    }
    let nr = to.len() as Ref;
    to.push(from[ru]);
    fwd[ru] = nr;
    nr
}

/// Collect `cells` in place, remapping every reference in `spine` and `roots`.
/// After it returns, `cells` holds only the live, compacted graph.
pub(crate) fn collect(cells: &mut Vec<Cell>, spine: &mut [Ref], roots: &mut [Ref]) {
    let from = std::mem::take(cells);
    let mut to: Vec<Cell> = Vec::with_capacity(from.len());
    let mut fwd = vec![u32::MAX; from.len()];

    for r in spine.iter_mut() {
        *r = forward(&from, &mut to, &mut fwd, *r);
    }
    for r in roots.iter_mut() {
        *r = forward(&from, &mut to, &mut fwd, *r);
    }

    // Cheney scan: `to` doubles as the work queue.
    let mut scan = 0;
    while scan < to.len() {
        let updated = match to[scan] {
            Cell::App(a, b) => {
                let a = forward(&from, &mut to, &mut fwd, a);
                let b = forward(&from, &mut to, &mut fwd, b);
                Cell::App(a, b)
            }
            Cell::Cons(h, t) => {
                let h = forward(&from, &mut to, &mut fwd, h);
                let t = forward(&from, &mut to, &mut fwd, t);
                Cell::Cons(h, t)
            }
            leaf => leaf, // Comb, Num, Input have no references; Ind never copied.
        };
        to[scan] = updated;
        scan += 1;
    }

    *cells = to;
}
