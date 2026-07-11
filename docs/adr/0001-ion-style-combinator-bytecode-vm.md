# Evaluation engine: ION-style combinator bytecode VM

## Context

Lazy K programs are pure SKI terms (all four notations reduce to `S`/`K`/`I` applications) evaluated with normal-order, lazy, shared semantics. Performance is an explicit goal.

## Decision

Compile the parsed combinator term into a compact, flat cell/instruction representation and reduce it on a custom **ION-style combinator VM** (flat array heap + explicit spine stack), rather than the reference interpreter's naive pointer-graph reduction, a classic G-machine, or a spineless TIM.

## Why

- The input is already SKI, so there are no user supercombinators to compile — a G-machine degenerates to `Unwind`-driven graph reduction, spending its machinery for little payoff.
- A flat-array heap with a spine stack gives cache-friendly, boxing-free node access in Rust and avoids native-stack overflow on deep spines.
- The model leaves room to later add optimized combinators (B, C, S′, …) and native-integer Church numerals without changing the execution substrate.

## Consequences

- We own the instruction encoding and reducer loop; correctness must be validated against tromp's reference test programs.
- Sharing is realized by in-place update of cells (overwriting a redex root with an indirection/result), so the heap layout must support cheap update and an indirection node.
