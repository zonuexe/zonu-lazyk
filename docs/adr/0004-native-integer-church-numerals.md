# Native integer cells for Church numerals at the I/O boundary

## Context

Output bytes are Church numerals forced by applying them to successor/zero; naively this costs O(value) reductions per byte, and input bytes must be supplied as numerals. I/O throughput is otherwise dominated by numeral construction/extraction.

## Decision

Two distinct native cell types at the I/O boundary:

- `Num(n)` — an input byte. It behaves as the Church numeral it stands for when
  applied: `Num(n) f x = f^n x`, unfolded one lazy layer per step
  (`n f x = f ((n-1) f x)`) so a single step allocates O(1).
- `Acc(k)` — the church2int counting accumulator, seeded as `Acc(0)`.

Extraction mirrors the reference interpreter's argument-swap protocol:

- `Inc x = x Inc` — Inc hands *itself* to its argument rather than forcing it.
- `Acc(k) Inc = Acc(k+1)` — the accumulator does the actual increment.
- church2int evaluates `head Inc Acc(0)` and reads the resulting `Acc(v)`.

EOF is a numeral `>= 256`.

## Why

The swap protocol is what makes higher-order uses of a numeral work: `Inc (K Inc)`
reduces to `(K Inc) Inc = Inc`, whereas a naive "force the argument to a number"
`Inc` gets stuck on any non-numeral argument. Keeping `Num` (a program's Church
numerals) and `Acc` (our private counter) as **distinct** types means the counter
rule can never collide with a program numeral.

## Consequences

- The reducer handles `Num`/`Acc` wherever a combinator could appear (application,
  forcing, GC copying); both are GC leaves.
- Only the I/O boundary is optimized; a program that arbitrarily transforms a byte
  still pays real reduction cost — we deliberately do not collapse arbitrary Church
  numerals in the graph.
- `Acc` is never exposed to the program (the parser emits only S/K/I; `Inc`/`Acc`
  enter solely via church2int), so there is no interaction with user terms.
