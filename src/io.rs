//! The I/O driver: bridges host byte streams and the combinator world (ADR-0004).
//!
//! Input is a lazy list of native `Num` cells built on demand from stdin
//! (see [`crate::vm::heap::Cell::Input`]). Output is read by treating the
//! program's result as a `\f. f head tail` list: `list K` is the head,
//! `list (K I)` is the tail. Each head is a Church numeral, forced to a value
//! with `head Inc 0`; a value `>= 256` ends the stream.
//!
//! The driver's live references (`list`, `head`) are held in [`crate::vm::Vm`]'s
//! root vector so they survive collections triggered inside `whnf`.

use crate::Error;
use crate::term::Comb;
use crate::vm::Vm;
use crate::vm::heap::Cell;
use std::io::{BufWriter, Write};

/// End-of-stream marker: a numeral `>= 256`.
pub const EOF: u32 = 256;

/// Drive the program (`vm.root`) over the installed input, writing output bytes.
pub fn drive<W: Write>(vm: &mut Vm, output: W) -> Result<(), Error> {
    let root = vm.root;
    let input = vm.alloc(Cell::Input);
    let list0 = vm.app(root, input);
    let list_slot = vm.roots.len();
    vm.roots.push(list0); // the current output list, preserved across GC.

    let mut out = BufWriter::new(output);
    let res = drive_loop(vm, list_slot, &mut out);
    vm.roots.truncate(list_slot);
    res?;
    out.flush()?;
    Ok(())
}

fn drive_loop<W: Write>(
    vm: &mut Vm,
    list_slot: usize,
    out: &mut BufWriter<W>,
) -> Result<(), Error> {
    loop {
        // head = list K
        let k = vm.alloc(Cell::Comb(Comb::K));
        let list = vm.roots[list_slot];
        let head_expr = vm.app(list, k);
        let head = vm.whnf(head_expr);

        // Protect `head` across the numeral extraction (which may collect).
        let head_slot = vm.roots.len();
        vm.roots.push(head);
        let value = numeral_value(vm, head_slot)?;
        vm.roots.truncate(head_slot);

        if value >= EOF {
            return Ok(());
        }
        out.write_all(&[value as u8])?;

        // tail = list (K I)
        let k = vm.alloc(Cell::Comb(Comb::K));
        let i = vm.alloc(Cell::Comb(Comb::I));
        let ki = vm.app(k, i);
        let list = vm.roots[list_slot];
        let tail = vm.app(list, ki);
        vm.roots[list_slot] = tail;
    }
}

/// Force the Church-numeral term at `roots[head_slot]` to its integer value via
/// `head Inc 0`.
fn numeral_value(vm: &mut Vm, head_slot: usize) -> Result<u32, Error> {
    let inc = vm.alloc(Cell::Comb(Comb::Inc));
    let zero = vm.alloc(Cell::Num(0));
    let head = vm.roots[head_slot];
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
