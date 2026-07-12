# Changelog

All notable changes to zonu-lazyk are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/), and this project adheres to
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **A curated embedding API** ([ADR-0006](docs/adr/0006-embedding-api.md)): `Program::compile(src)` parses and optimizes once, then `run`/`run_with` stream to any `Write` and `eval`/`eval_with` return a `Vec<u8>` — compile once, run many times. The pipeline modules (`parser`, `vm`, …) are now documented as unstable internals rather than the embedding contract; while the crate is `0.x`, `Program`/`Limits`/`Error` may change between minor versions.
- **Opt-in resource limits for untrusted programs** ([ADR-0007](docs/adr/0007-embedding-hardening.md)): `Limits { max_steps, max_output_bytes }` bound one run. `max_steps` stops a non-terminating program with `Error::StepLimit`; `max_output_bytes` stops an unbounded output stream with `Error::OutputLimit`. Both default to unlimited, and with no limit the step check is a single comparison per reduction.

### Changed

- `Error` now implements `Display` and `std::error::Error` (with a `source()` chain for parse and I/O errors), so embedders can use `?`, `Box<dyn Error>`, and `{}` normally. The CLI prints errors via `Display`.

## [0.1.0] - 2026-07-12

### Added

- A Lazy K interpreter. `zonu-lazyk <program-file>` reads the program from a file, streams standard input as the input byte stream, and writes the output byte stream to standard output; a numeral `>= 256` ends the stream. See <https://tromp.github.io/cl/lazy-k.html>.
- All four notations, freely mixable, with `#` comments: Combinatory Logic (`S`/`K`/`I` + parens), Unlambda (`` ` `` application), Iota (`*` application, with `i` as the iota combinator when it is a direct operand of `*`), and Jot (`0`/`1`, one number spanning any whitespace and comments). Behaviour matches the reference interpreter across every notation — verified by differential fuzzing (0 mismatches over thousands of programs) and by running its `reverse`, `rot13`, and `hello` example programs.
- Embedding: `zonu_lazyk::run(program_src, input, output)` runs a program against any `Read`/`Write` pair.
- Performance: an ION-style combinator VM with extended combinators (`B`/`C`/`S'`/`B'`/`C'`) introduced by a peephole optimizer, native integer numerals with O(1) church2int extraction, 8-byte packed heap cells, and a Cheney copying collector that bounds memory on unbounded streams. On 20 KB inputs it runs on par with the reference interpreter. Design decisions are recorded in `docs/adr/`.

[Unreleased]: https://github.com/zonuexe/zonu-lazyk/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/zonuexe/zonu-lazyk/releases/tag/v0.1.0
