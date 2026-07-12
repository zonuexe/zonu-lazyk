# Embedding: a curated library API

## Context

zonu-lazyk already builds a `[lib]`, so a Rust program can `use zonu_lazyk::…`.
But the only real entry point is `run(src, input, output)`, which reparses and
re-optimizes on every call, and every internal module (`parser`, `term`, `vm`,
`compile`, `io`) is `pub` — freezing internals as a public contract.

## Decision

Expose a small, intentional surface at the crate root and keep the internals as
implementation detail:

- **`Program`** — a compiled program. `Program::compile(src) -> Result<_, ParseError>`
  parses and optimizes once; `run`/`run_with` stream to any `Write`, and
  `eval`/`eval_with` return a `Vec<u8>`. Compile once, run many times.
- **`Limits`** — opt-in resource bounds (see ADR-0007).
- **`Error`** and **`ParseError`** re-exported at the root.
- The free `run(src, input, output)` stays as the one-shot convenience.
- `parser`/`term`/`vm`/`compile`/`io` remain `pub` (the CLI, tests, and benches
  use them) but are `#[doc(hidden)]` and documented as unstable internals — not
  the embedding contract.

`src/main.rs` is a thin client of `Program`.

## Pre-1.0: the embedding API is unstable

While zonu-lazyk is `0.x`, `Program`/`Limits`/`Error` may change between minor
versions; embedders should pin an exact version. Stated in the crate docs and the
README, not enforced.

## Consequences

- The curated types are what we commit to keeping stable; internals can move.
- `Program` holds only the compiled `Term`; each `run` loads a fresh VM heap, so
  runs are independent and a `Program` is cheap to reuse and `Send`-able.
