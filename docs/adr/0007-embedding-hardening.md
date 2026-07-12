# Embedding hardening: a real error type and resource limits

## Context

[ADR-0006](0006-embedding-api.md) curated the embedding API but left two sharp
edges: `Error` is `Debug`-only (awkward to propagate or display), and a Lazy K
program can loop forever or emit an unbounded output stream — an embedder running
untrusted programs cannot bound them. This mirrors niiLISP's ADR-0040.

## `Error` implements `Display` + `std::error::Error`

So embedders use `?`, `Box<dyn Error>`, and `{}` normally. `Error::Parse` wraps
`ParseError`; the `source()` chain is preserved for `Parse` and `Io`.

## Opt-in resource limits for untrusted programs

`Limits { max_steps, max_output_bytes }` (both `Option`, default `None` = no
limit):

- **`max_steps`** caps total reduction steps for one run. The reducer counts each
  `whnf` loop iteration; on overflow it stops and the run returns
  `Err(Error::StepLimit)`. This is what bounds a non-terminating program (one that
  never produces the next output byte).
- **`max_output_bytes`** caps how many bytes a run may emit; the I/O driver stops
  at the ceiling with `Err(Error::OutputLimit)`. This bounds a program with
  well-formed but infinite output (e.g. a `repeat` stream).

Both are checked per run (a fresh VM per `run`), so a limit bounds one run, not a
`Program`'s lifetime.

- **Rejected: a wall-clock timeout thread.** A step counter is deterministic,
  portable, and reproducible — the same program always stops at the same point —
  and needs no thread or interruption of synchronous Rust.
- **Cost:** with no limit the step check is one comparison against `u64::MAX` per
  reduction (no counting); off by default, so it is a safety valve for untrusted
  input, not an always-on tax.

## Consequences

- `Error` gains `StepLimit` and `OutputLimit` variants.
- The VM carries a step counter and ceiling; the I/O driver carries an output
  ceiling. Neither is observable unless a limit is set.
