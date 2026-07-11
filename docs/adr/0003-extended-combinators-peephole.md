# Compile-time peephole optimization into extended combinators

## Context

Lazy K input is pure SKI, typically produced by a lambda→SKI compiler, which yields large graphs with many `S(K·)·` patterns. Reduction-step count dominates runtime.

## Decision

After parsing, run a compile-time peephole pass that rewrites local SKI patterns into an **extended combinator set** (`B`, `C`, `S'`, `B'`, `C'`) before execution, e.g. `S(Kp)(Kq) → K(pq)`, `S(Kp)I → p`, `S(Kp)q → B p q`, `Sp(Kq) → C p q`. The VM implements rewrite rules for these combinators directly.

## Why

These are standard Turner-style combinator optimizations that cut reduction steps substantially while preserving observable behaviour (the rewrites are extensional equalities). They are the primary step-count lever and compose with the ION VM (ADR-0001).

## Consequences

- The VM's instruction/rewrite set grows beyond S/K/I; each extended combinator needs a correct rule and test coverage.
- Correctness must be validated against tromp's reference programs, since a wrong rewrite silently changes results.
