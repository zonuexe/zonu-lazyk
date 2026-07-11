# Native integer cells for Church numerals at the I/O boundary

## Context

Output bytes are Church numerals forced by applying them to successor/zero; naively this costs O(value) reductions per byte, and input bytes must be supplied as numerals. I/O throughput is otherwise dominated by numeral construction/extraction.

## Decision

Add a native integer cell type (`Num`) and primitive combinators (`inc`, `zero`, …). Precompute the input numerals `0..=256` as a cached table fed into the input stream; extract output numerals with a native counter fast path. EOF is a numeral `>= 256`.

## Why

A native `Num` behaves identically to the corresponding Church numeral when applied, so semantics are preserved, but I/O-boundary construction and extraction become O(1)/O(value-on-native-counter) instead of driving combinator reductions. This is the second major performance lever alongside ADR-0003.

## Consequences

- The reducer must handle `Num` cells wherever a combinator could appear (application to `Num`, forcing, GC copying).
- Only the I/O boundary is optimized; a program that arbitrarily transforms a byte still pays real reduction cost — deliberately not collapsing arbitrary Church numerals in the graph (that would risk sharing/semantics).
