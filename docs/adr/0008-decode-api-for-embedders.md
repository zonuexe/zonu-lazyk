# Decode API for embedders: numerals, value lists, and term input

## Context

An embedder (the λ-1 translator in `lambda1-grandprix`, ADR-0003 there) decodes
a program's *result value*, but 0.2.0 only exposed byte-stream I/O. That forced
workarounds: a Church numeral was decoded by making the program emit N bytes and
counting them (`decodeInt`), and structured values had no path at all. This ADR
adds a decode surface so those observations are direct.

## Decisions

- **Numeral extraction** (their priority 1). `Program::eval_numeral()` reduces the
  program term itself as a Church numeral and returns a `u64`, with no 256 cap.
  The accumulator cell `Acc` widens from `u32` to `u64` (61 bits in the packed
  layout). Removes the "emit N bytes and count" workaround.
- **Raw value list** (their priorities 2-flat and 3). `Program::eval_values(input,
  &DecodeOptions)` returns the output `\f. f h t` list as `Vec<u64>` — raw Church
  numerals, not truncated to bytes. `DecodeOptions { eof, max_values, max_steps }`
  makes the EOF sentinel configurable/disable-able (default `Some(256)`) and
  bounds the read. Covers strings and integer arrays.
- **Term input** (their priority 4). `Program::from_term(Term)` accepts an
  already-built AST, and `Term`/`Comb` are re-exported at the crate root, so a
  translator that already builds SKI terms skips the render-to-string / re-parse
  trip.
- **Host int → Church numeral** (their priority 6). `church_numeral(n)` returns a
  native numeral term (`Term::Num`), O(1), that behaves as `\f x. f^n x`.

Their priorities 5 (bounded evaluation) and 7 (embedding-safe failure) were
already met by ADR-0007 (`Limits`, `Error::StepLimit`, `Result`-based errors).

## Not done here

Full structural decode of nested Scott lists (their priority 2 for `decodeJson`)
needs term-handle traversal across the VM heap — a larger design. `eval_values`
covers flat lists (strings, integer arrays); nested-structure decode is deferred.

## Consequences

- `Acc` is `u64`; byte-stream I/O is unchanged (differential-fuzzed at 0
  mismatches). The extra decode entry points do not touch the Lazy K language or
  the demo output format.
- `Term`/`Comb` become part of the (still `0.x`-unstable) public surface.
