//! The I/O driver: bridges host byte streams and the combinator world (ADR-0004).
//!
//! Input is a lazy list of native `Num` cells built on demand from stdin
//! (see [`crate::vm::heap::Cell::Input`]). Output is read by treating the
//! program's result as a `\f. f head tail` list: `list K` is the head,
//! `list (K I)` is the tail. Each head is a Church numeral, forced to a value
//! with `head Inc 0`; a value `>= 256` ends the stream.
//!
//! Besides the byte-stream [`drive`], the decode helpers [`collect_values`] and
//! [`eval_numeral`] expose the raw Church numerals for embedders (ADR-0008).
//!
//! The driver's live references (`list`, `head`) are held in [`crate::vm::Vm`]'s
//! root vector so they survive collections triggered inside `whnf`.

use crate::Error;
use crate::term::Comb;
use crate::vm::Vm;
use crate::vm::heap::Cell;
use std::io::{BufWriter, Write};

/// The default end-of-stream marker: a numeral `>= 256`.
pub const EOF: u64 = 256;

/// Apply `vm.root` to a fresh lazy input list and root the result; returns the
/// root-vector slot holding the current output list.
fn setup_output_list(vm: &mut Vm) -> usize {
    let root = vm.root;
    let input = vm.alloc(Cell::Input);
    let list0 = vm.app(root, input);
    let slot = vm.roots.len();
    vm.roots.push(list0);
    slot
}

/// The value of the current head: `list K` forced to a Church numeral.
fn head_value(vm: &mut Vm, list_slot: usize) -> Result<u64, Error> {
    let k = vm.alloc(Cell::Comb(Comb::K));
    let list = vm.roots[list_slot];
    let head_expr = vm.app(list, k);
    let head = vm.whnf(head_expr);
    if vm.step_limit_hit {
        return Err(Error::StepLimit);
    }
    let head_slot = vm.roots.len();
    vm.roots.push(head);
    let value = numeral_value(vm, head_slot)?;
    vm.roots.truncate(head_slot);
    Ok(value)
}

/// Replace the current list with its tail: `list (K I)`.
fn advance_tail(vm: &mut Vm, list_slot: usize) {
    let k = vm.alloc(Cell::Comb(Comb::K));
    let i = vm.alloc(Cell::Comb(Comb::I));
    let ki = vm.app(k, i);
    let list = vm.roots[list_slot];
    let tail = vm.app(list, ki);
    vm.roots[list_slot] = tail;
}

/// Drive the program (`vm.root`) over the installed input, writing output bytes.
/// EOF is a numeral `>= 256`; each byte is `value as u8`.
pub fn drive<W: Write>(vm: &mut Vm, output: W) -> Result<(), Error> {
    let list_slot = setup_output_list(vm);
    let mut out = BufWriter::new(output);
    let res = (|| {
        let mut written: u64 = 0;
        loop {
            let value = head_value(vm, list_slot)?;
            if value >= EOF {
                return Ok(());
            }
            if written >= vm.max_output {
                return Err(Error::OutputLimit);
            }
            out.write_all(&[value as u8])?;
            written += 1;
            advance_tail(vm, list_slot);
        }
    })();
    vm.roots.truncate(list_slot);
    res?;
    out.flush()?;
    Ok(())
}

/// Collect the output list as raw Church-numeral values (ADR-0008), without the
/// `u8` truncation. Stops — returning the values so far — at the first numeral
/// `>= eof` (when `eof` is `Some`) or after `max_values` elements (when `Some`).
/// `max_steps` (via `Vm`) still bounds divergence with `Error::StepLimit`. With
/// both `eof` and `max_values` `None`, an unbounded stream needs `max_steps`.
pub fn collect_values(
    vm: &mut Vm,
    eof: Option<u64>,
    max_values: Option<u64>,
) -> Result<Vec<u64>, Error> {
    let list_slot = setup_output_list(vm);
    let res = (|| {
        let mut values = Vec::new();
        loop {
            if let Some(n) = max_values
                && values.len() as u64 >= n
            {
                return Ok(values);
            }
            let value = head_value(vm, list_slot)?;
            if let Some(e) = eof
                && value >= e
            {
                return Ok(values);
            }
            values.push(value);
            advance_tail(vm, list_slot);
        }
    })();
    vm.roots.truncate(list_slot);
    res
}

/// Evaluate `vm.root` itself as a Church numeral and return its value (ADR-0008),
/// with no 256 cap. The program term is applied to `Inc`/`Acc(0)` directly — no
/// input list, no byte stream.
pub fn eval_numeral(vm: &mut Vm) -> Result<u64, Error> {
    let root = vm.root;
    let inc = vm.alloc(Cell::Comb(Comb::Inc));
    let acc0 = vm.alloc(Cell::Acc(0));
    let e = vm.app(root, inc);
    let e = vm.app(e, acc0);
    let r = vm.whnf(e);
    if vm.step_limit_hit {
        return Err(Error::StepLimit);
    }
    match vm.heap.get(r) {
        Cell::Acc(v) => Ok(v),
        other => Err(Error::IllFormedOutput(format!(
            "term is not a Church numeral (reduced to {other:?})"
        ))),
    }
}

/// Force the Church-numeral term at `roots[head_slot]` to its integer value via
/// `head Inc Acc(0)`.
fn numeral_value(vm: &mut Vm, head_slot: usize) -> Result<u64, Error> {
    let inc = vm.alloc(Cell::Comb(Comb::Inc));
    let zero = vm.alloc(Cell::Acc(0));
    let head = vm.roots[head_slot];
    let e = vm.app(head, inc);
    let e = vm.app(e, zero);
    let r = vm.whnf(e);
    if vm.step_limit_hit {
        return Err(Error::StepLimit);
    }
    match vm.heap.get(r) {
        Cell::Acc(v) => Ok(v),
        other => Err(Error::IllFormedOutput(format!(
            "output list element is not a Church numeral (reduced to {other:?})"
        ))),
    }
}
