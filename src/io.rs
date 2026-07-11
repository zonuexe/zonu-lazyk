//! The I/O driver: bridges host byte streams and the combinator world (ADR-0004).
//!
//! Input is a lazy list of native `Num` cells built on demand from stdin
//! (see [`crate::vm::heap::Cell::Input`]). Output is read by treating the
//! program's result as a `\f. f head tail` list: `list K` is the head,
//! `list (K I)` is the tail. Each head is a Church numeral, forced to a value
//! with `head Inc 0`; a value `>= 256` ends the stream.

use crate::Error;
use crate::term::Comb;
use crate::vm::Vm;
use crate::vm::heap::{Cell, Ref};
use std::io::Write;

/// End-of-stream marker: a numeral `>= 256`.
pub const EOF: u32 = 256;

/// Drive the program (`vm.root`) over the installed input, writing output bytes.
pub fn drive<W: Write>(vm: &mut Vm, output: W) -> Result<(), Error> {
    let root = vm.root;
    let input = vm.alloc(Cell::Input);
    // The output stream is `program input`.
    let mut list = vm.app(root, input);
    let mut out = std::io::BufWriter::new(output);

    loop {
        // head = list K
        let k = vm.alloc(Cell::Comb(Comb::K));
        let head_expr = vm.app(list, k);
        let head = vm.whnf(head_expr);

        let value = numeral_value(vm, head)?;
        if value >= EOF {
            break;
        }
        out.write_all(&[value as u8])?;

        // tail = list (K I)
        let k = vm.alloc(Cell::Comb(Comb::K));
        let i = vm.alloc(Cell::Comb(Comb::I));
        let ki = vm.app(k, i);
        list = vm.app(list, ki);
    }

    out.flush()?;
    Ok(())
}

/// Force a Church-numeral term to its integer value via `head Inc 0`.
fn numeral_value(vm: &mut Vm, head: Ref) -> Result<u32, Error> {
    let inc = vm.alloc(Cell::Comb(Comb::Inc));
    let zero = vm.alloc(Cell::Num(0));
    let e = vm.app(head, inc);
    let e = vm.app(e, zero);
    let r = vm.whnf(e);
    match vm.heap.get(r) {
        Cell::Num(v) => Ok(v),
        other => Err(Error::IllFormedOutput(format!(
            "output list element is not a Church numeral (reduced to {other:?})"
        ))),
    }
}
