//! Normal-order reduction to WHNF via an explicit spine stack (ADR-0001).
//!
//! The spine lives in `Vm::spine` so the collector can see and relocate it.
//! Each call marks a `base` (the caller's spine height) and restores it on
//! return. A single loop iteration allocates O(1) cells, and the collector runs
//! only at the top-of-loop safe point (never inside `alloc`), so cached spine
//! indices stay valid within an iteration.

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

    /// Follow any `Ind` chain from `r` and return the cell it resolves to.
    #[inline]
    fn resolve(&self, mut r: Ref) -> Cell {
        loop {
            match self.heap.get(r) {
                Cell::Ind(t) => r = t,
                cell => return cell,
            }
        }
    }

    /// Reduce the cell reachable from `root` to weak head normal form and return
    /// a reference to the resulting (non-`Ind`) cell.
    pub fn whnf(&mut self, root: Ref) -> Ref {
        let base = self.spine.len();
        let mut node = root;

        loop {
            // Reduction-step ceiling (ADR-0007). Unlimited unless a limit is set,
            // so the common path is one comparison against `u64::MAX`.
            if self.max_steps != u64::MAX {
                self.steps += 1;
                if self.steps > self.max_steps {
                    self.step_limit_hit = true;
                    return self.finish(base, node);
                }
            }

            // Safe point: collect if the heap has grown enough. `node` is the
            // only live reference not already in `spine`/`roots`, so protect it.
            if self.heap.len() >= self.gc_threshold {
                self.roots.push(node);
                self.gc();
                node = self.roots.pop().unwrap();
            }

            match self.heap.get(node) {
                Cell::Ind(target) => node = target,

                Cell::App(f, _) => {
                    self.spine.push(node);
                    node = f;
                }

                Cell::Num(nv) => {
                    // A native `Num(n)` is the Church numeral it stands for
                    // (ADR-0004): `n f x = f ((n-1) f x)`, unfolded one lazy
                    // layer per step. Fewer than two args ⇒ a value.
                    if self.spine.len() - base < 2 {
                        return self.finish(base, node);
                    }
                    let sl = self.spine.len();
                    let f = self.arg(self.spine[sl - 1]);
                    let x = self.arg(self.spine[sl - 2]);
                    let redex = self.spine[sl - 2];
                    // Fast path: `Num(n) Inc Acc(k) = Acc(k+n)`, the church2int
                    // extraction. Collapses O(n) counting into one step. Inc/Acc
                    // never appear in user terms, so this only fires at the I/O
                    // boundary.
                    if matches!(self.resolve(f), Cell::Comb(Comb::Inc))
                        && let Cell::Acc(k) = self.resolve(x)
                    {
                        let acc = self.alloc(Cell::Acc(k + nv));
                        self.heap.set(redex, Cell::Ind(acc));
                    } else if nv == 0 {
                        self.heap.set(redex, Cell::Ind(x)); // 0 f x = x
                    } else {
                        // n f x = f ((n-1) f x)
                        let pred = self.alloc(Cell::Num(nv - 1));
                        let pf = self.app(pred, f);
                        let pfx = self.app(pf, x);
                        let res = self.app(f, pfx);
                        self.heap.set(redex, Cell::Ind(res));
                    }
                    self.spine.truncate(sl - 2);
                    node = redex;
                }

                // The counting accumulator: `Acc(k) Inc = Acc(k+1)`. It is only
                // ever produced/consumed by church2int and only meets `Inc`.
                Cell::Acc(k) => {
                    if self.spine.len() == base {
                        return node; // a bare accumulator is the final value.
                    }
                    let redex = self.spine[self.spine.len() - 1];
                    let arg = self.arg(redex);
                    if matches!(self.resolve(arg), Cell::Comb(Comb::Inc)) {
                        let next = self.alloc(Cell::Acc(k + 1));
                        self.heap.set(redex, Cell::Ind(next));
                        self.spine.pop();
                        node = redex;
                    } else {
                        // Applied to anything but Inc — ill-formed output.
                        return self.finish(base, node);
                    }
                }

                Cell::Input => {
                    let byte = self.read_input_byte();
                    let head = self.alloc(Cell::Num(byte));
                    let tail = self.alloc(Cell::Input);
                    self.heap.set(node, Cell::Cons(head, tail));
                    // Loop again; `node` is now a Cons.
                }

                Cell::Cons(h, t) => {
                    if self.spine.len() == base {
                        return node; // bare list cell is a value.
                    }
                    // Cons h t applied to f  ==>  f h t
                    let redex = self.spine[self.spine.len() - 1];
                    let f = self.arg(redex);
                    let fh = self.app(f, h);
                    self.heap.set(redex, Cell::App(fh, t));
                    self.spine.pop();
                    node = redex;
                }

                Cell::Comb(c) => {
                    let ar = c.arity();
                    if self.spine.len() - base < ar {
                        return self.finish(base, node);
                    }
                    let sl = self.spine.len();
                    let redex = self.spine[sl - ar];

                    match c {
                        // I x = x
                        Comb::I => {
                            let x = self.arg(self.spine[sl - 1]);
                            self.heap.set(redex, Cell::Ind(x));
                        }
                        // K x y = x
                        Comb::K => {
                            let x = self.arg(self.spine[sl - 1]);
                            self.heap.set(redex, Cell::Ind(x));
                        }
                        // S f g x = (f x) (g x)
                        Comb::S => {
                            let f = self.arg(self.spine[sl - 1]);
                            let g = self.arg(self.spine[sl - 2]);
                            let x = self.arg(self.spine[sl - 3]);
                            let fx = self.app(f, x);
                            let gx = self.app(g, x);
                            self.heap.set(redex, Cell::App(fx, gx));
                        }
                        // B f g x = f (g x)
                        Comb::B => {
                            let f = self.arg(self.spine[sl - 1]);
                            let g = self.arg(self.spine[sl - 2]);
                            let x = self.arg(self.spine[sl - 3]);
                            let gx = self.app(g, x);
                            self.heap.set(redex, Cell::App(f, gx));
                        }
                        // C f g x = (f x) g
                        Comb::C => {
                            let f = self.arg(self.spine[sl - 1]);
                            let g = self.arg(self.spine[sl - 2]);
                            let x = self.arg(self.spine[sl - 3]);
                            let fx = self.app(f, x);
                            self.heap.set(redex, Cell::App(fx, g));
                        }
                        // S' c f g x = c (f x) (g x)
                        Comb::Sp => {
                            let cc = self.arg(self.spine[sl - 1]);
                            let f = self.arg(self.spine[sl - 2]);
                            let g = self.arg(self.spine[sl - 3]);
                            let x = self.arg(self.spine[sl - 4]);
                            let fx = self.app(f, x);
                            let gx = self.app(g, x);
                            let cfx = self.app(cc, fx);
                            self.heap.set(redex, Cell::App(cfx, gx));
                        }
                        // B' c f g x = c f (g x)
                        Comb::Bp => {
                            let cc = self.arg(self.spine[sl - 1]);
                            let f = self.arg(self.spine[sl - 2]);
                            let g = self.arg(self.spine[sl - 3]);
                            let x = self.arg(self.spine[sl - 4]);
                            let gx = self.app(g, x);
                            let cf = self.app(cc, f);
                            self.heap.set(redex, Cell::App(cf, gx));
                        }
                        // C' c f g x = c (f x) g
                        Comb::Cp => {
                            let cc = self.arg(self.spine[sl - 1]);
                            let f = self.arg(self.spine[sl - 2]);
                            let g = self.arg(self.spine[sl - 3]);
                            let x = self.arg(self.spine[sl - 4]);
                            let fx = self.app(f, x);
                            let cfx = self.app(cc, fx);
                            self.heap.set(redex, Cell::App(cfx, g));
                        }
                        // Inc x = x Inc (the reference's argument-swap trick).
                        // Delegates counting to the argument, which handles the
                        // higher-order cases the naive "force to a number" cannot.
                        // `node` is the Inc cell itself; reuse it as the operand.
                        Comb::Inc => {
                            let x = self.arg(self.spine[sl - 1]);
                            self.heap.set(redex, Cell::App(x, node));
                        }
                    }

                    self.spine.truncate(sl - ar);
                    node = redex;
                }
            }
        }
    }

    /// Pop this call's spine frame and return the WHNF reference.
    #[inline]
    fn finish(&mut self, base: usize, node: Ref) -> Ref {
        let result = if self.spine.len() > base {
            self.spine[base]
        } else {
            node
        };
        self.spine.truncate(base);
        result
    }
}
