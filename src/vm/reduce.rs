//! Normal-order reduction to WHNF via an explicit spine stack (ADR-0001).
//!
//! Unwind the leftmost spine, and when the head is a combinator with `arity()`
//! arguments available, fire its rewrite rule and overwrite the redex root with
//! the result (sharing). `Input` cells read one byte on first force and rewrite
//! themselves into a `Cons`, so input is lazy and read in order exactly once.

use super::Vm;
use super::heap::{Cell, Ref};
use crate::term::Comb;

impl Vm {
    /// Operand of an application cell.
    #[inline]
    fn arg(&self, app: Ref) -> Ref {
        match self.heap.get(app) {
            Cell::App(_, x) => x,
            other => unreachable!("expected App on the spine, found {other:?}"),
        }
    }

    /// Reduce the cell reachable from `root` to weak head normal form and return
    /// a reference to the resulting (non-`Ind`) cell.
    pub fn whnf(&mut self, root: Ref) -> Ref {
        let mut spine: Vec<Ref> = Vec::new();
        let mut node = root;

        loop {
            match self.heap.get(node) {
                Cell::Ind(target) => node = target,

                Cell::App(f, _) => {
                    spine.push(node);
                    node = f;
                }

                // A native `Num(n)` behaves as the Church numeral it stands for
                // (ADR-0004): applied to `f` and `x` it yields `f^n x`. Unfold
                // one layer lazily — `n f x = f ((n-1) f x)` — so a single step
                // allocates O(1), never O(n). With fewer than two arguments it
                // is a value in WHNF.
                Cell::Num(nv) => {
                    let n = spine.len();
                    if n < 2 {
                        return spine.first().copied().unwrap_or(node);
                    }
                    let f = self.arg(spine[n - 1]);
                    let x = self.arg(spine[n - 2]);
                    let redex = spine[n - 2];
                    if nv == 0 {
                        self.heap.set(redex, Cell::Ind(x)); // 0 f x = x
                    } else {
                        let pred = self.alloc(Cell::Num(nv - 1));
                        let pf = self.app(pred, f);
                        let pfx = self.app(pf, x); // (n-1) f x
                        let res = self.app(f, pfx); // f ((n-1) f x)
                        self.heap.set(redex, Cell::Ind(res));
                    }
                    spine.truncate(n - 2);
                    node = redex;
                }

                Cell::Input => {
                    let byte = self.read_input_byte();
                    let head = self.alloc(Cell::Num(byte));
                    let tail = self.alloc(Cell::Input);
                    self.heap.set(node, Cell::Cons(head, tail));
                    // loop again; `node` is now a Cons.
                }

                Cell::Cons(h, t) => {
                    if spine.is_empty() {
                        return node; // a bare list cell is a value.
                    }
                    // Cons h t applied to f  ==>  f h t
                    let redex = spine[spine.len() - 1];
                    let f = self.arg(redex);
                    let fh = self.app(f, h);
                    self.heap.set(redex, Cell::App(fh, t));
                    spine.pop();
                    node = redex;
                }

                Cell::Comb(c) => {
                    let ar = c.arity();
                    if spine.len() < ar {
                        return spine.first().copied().unwrap_or(node);
                    }
                    let n = spine.len();
                    let redex = spine[n - ar];

                    match c {
                        // I x = x
                        Comb::I => {
                            let x = self.arg(spine[n - 1]);
                            self.heap.set(redex, Cell::Ind(x));
                        }
                        // K x y = x
                        Comb::K => {
                            let x = self.arg(spine[n - 1]);
                            self.heap.set(redex, Cell::Ind(x));
                        }
                        // S f g x = (f x) (g x)
                        Comb::S => {
                            let f = self.arg(spine[n - 1]);
                            let g = self.arg(spine[n - 2]);
                            let x = self.arg(spine[n - 3]);
                            let fx = self.app(f, x);
                            let gx = self.app(g, x);
                            self.heap.set(redex, Cell::App(fx, gx));
                        }
                        // B f g x = f (g x)
                        Comb::B => {
                            let f = self.arg(spine[n - 1]);
                            let g = self.arg(spine[n - 2]);
                            let x = self.arg(spine[n - 3]);
                            let gx = self.app(g, x);
                            self.heap.set(redex, Cell::App(f, gx));
                        }
                        // C f g x = (f x) g
                        Comb::C => {
                            let f = self.arg(spine[n - 1]);
                            let g = self.arg(spine[n - 2]);
                            let x = self.arg(spine[n - 3]);
                            let fx = self.app(f, x);
                            self.heap.set(redex, Cell::App(fx, g));
                        }
                        // S' c f g x = c (f x) (g x)
                        Comb::Sp => {
                            let cc = self.arg(spine[n - 1]);
                            let f = self.arg(spine[n - 2]);
                            let g = self.arg(spine[n - 3]);
                            let x = self.arg(spine[n - 4]);
                            let fx = self.app(f, x);
                            let gx = self.app(g, x);
                            let cfx = self.app(cc, fx);
                            self.heap.set(redex, Cell::App(cfx, gx));
                        }
                        // B' c f g x = c f (g x)
                        Comb::Bp => {
                            let cc = self.arg(spine[n - 1]);
                            let f = self.arg(spine[n - 2]);
                            let g = self.arg(spine[n - 3]);
                            let x = self.arg(spine[n - 4]);
                            let gx = self.app(g, x);
                            let cf = self.app(cc, f);
                            self.heap.set(redex, Cell::App(cf, gx));
                        }
                        // C' c f g x = c (f x) g
                        Comb::Cp => {
                            let cc = self.arg(spine[n - 1]);
                            let f = self.arg(spine[n - 2]);
                            let g = self.arg(spine[n - 3]);
                            let x = self.arg(spine[n - 4]);
                            let fx = self.app(f, x);
                            let cfx = self.app(cc, fx);
                            self.heap.set(redex, Cell::App(cfx, g));
                        }
                        // Inc n = n + 1  (native successor at the I/O boundary)
                        Comb::Inc => {
                            let x = self.arg(spine[n - 1]);
                            let xw = self.whnf(x);
                            match self.heap.get(xw) {
                                Cell::Num(k) => {
                                    let next = self.alloc(Cell::Num(k + 1));
                                    self.heap.set(redex, Cell::Ind(next));
                                }
                                // Inc of a non-number is stuck: expose it as WHNF.
                                _ => return spine.first().copied().unwrap_or(node),
                            }
                        }
                    }

                    spine.truncate(n - ar);
                    node = redex;
                }
            }
        }
    }
}
