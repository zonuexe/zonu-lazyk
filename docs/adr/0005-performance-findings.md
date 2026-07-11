# Performance findings and optimization roadmap

## Context

Performance is a project goal. This records what profiling found so future work
targets the right thing instead of re-deriving it.

## Measurements

Workload for a 2000-byte input (steps = whnf loop iterations):

| program | steps    | allocs   | GCs | vs reference (20 KB) |
| ------- | -------- | -------- | --- | -------------------- |
| cat     | 46 k     | 30 k     | 0   | —                    |
| rot13   | 8.75 M   | 2.21 M   | 34  | ~1.5x slower         |
| reverse | 10.68 M  | 3.11 M   | 52  | ~1.8x slower         |

- ~2.3 ns per reduction step; the hot programs are **step-bound**, doing
  thousands of steps per byte because Lazy K numerals are unary.
- `cat` is fast because church2int extraction is O(1) (ADR-0004 fast path).
- allocs are ~0.28 per step; `Cell` is 12 bytes.

## What was tried

- **church2int O(1) extraction** (`Num Inc Acc = Acc(k+n)`): kept — `cat` went
  from ~0.8 MiB/s to ~18 MiB/s (~22x).
- **`get_unchecked` heap access**: reverted — ~1% (within noise), not worth the
  `unsafe`. Bounds checks are not the bottleneck.
- **Interning `Num(0..=256)`**: reverted — net negative. Bump allocation is
  already near-free, and pinning 257 always-live cells added GC copying.

## Decision / roadmap

The remaining `rot13`/`reverse` gap to the reference is not allocation or bounds
checks; it is per-step memory traffic and cache behaviour. Ranked next steps:

1. **Shrink `Cell` 12 → 8 bytes** — DONE. Storage is now a packed `u64` (tag in
   the top 3 bits; `App`/`Cons` hold two 30-bit refs; `Cell` stays the logical
   view via encode/decode in `heap`/`gc`). Result: `cat` −25%, `rot13` −34%,
   `reverse` −39%. This closed almost the whole gap — on 20 KB inputs `rot13` now
   matches the reference and `reverse` is within ~1.1x.
2. **Threaded / spineless spine.** The reference threads the spine through the
   graph's operator field instead of a side `Vec`; better locality. Larger change,
   now lower priority given (1)'s result.
3. **Deeper combinator optimization** to cut step count (director strings, or a
   richer peephole) — helps compute-bound programs like `rot13`.

The interpreter is correct and reference-compatible across all four notations and
now performs on par with the reference; the SKI-focused release does not depend
on further optimization.
