//! The I/O driver: bridges the host byte streams and the combinator world
//! (ADR-0004).
//!
//! Input: a lazy cons-list whose elements are native `Num` cells, one per byte,
//! terminated by 256. Only materialized as the program forces it.
//!
//! Output: force the program's result list; for each element, force it to a
//! numeral, emit `value as u8`, and stop once a numeral `>= 256` (EOF) appears.

use crate::vm::Vm;

/// Precomputed input numerals `0..=256` are injected as native `Num` cells.
pub const EOF: u32 = 256;

/// Drive the program `vm.root` over `input`, writing bytes to `output`.
pub fn drive<R: std::io::Read, W: std::io::Write>(
    vm: &mut Vm,
    input: R,
    output: W,
) -> Result<(), crate::Error> {
    // TODO:
    //   - wrap `input` in a BufReader; build the lazy input list on demand
    //   - apply `vm.root` to the input list
    //   - loop: force head to a numeral via `whnf`; if >= EOF stop; else write byte
    //   - force tail; flush `output` (BufWriter) at the end
    let _ = (vm, input, output);
    todo!("io::drive")
}
