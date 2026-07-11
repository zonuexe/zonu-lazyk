//! Cheney two-space copying collector (ADR-0002).
//!
//! Roots are the spine stack plus the live I/O cursors. Live cells are copied
//! into to-space and compacted; `Ind` chains are shortcut while copying.

use super::heap::Ref;

/// Collect, given the current roots. Returns the relocated roots in order.
///
/// TODO:
///   1. flip semi-spaces; reset the to-space bump pointer
///   2. `forward` each root: copy if not already, leaving a forwarding `Ind`
///   3. scan to-space breadth-first (Cheney), forwarding children of App cells
///   4. shortcut indirections encountered during forwarding
pub fn collect(_roots: &mut [Ref]) {
    todo!("gc::collect")
}
