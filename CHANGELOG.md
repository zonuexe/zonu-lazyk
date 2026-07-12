# Changelog

All notable changes to zonu-lazyk are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/), and this project adheres to
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- A Lazy K interpreter. `zonu-lazyk <program-file>` reads the program from a file, streams standard input as the input byte stream, and writes the output byte stream to standard output; a numeral `>= 256` ends the stream. See <https://tromp.github.io/cl/lazy-k.html>.
- All four notations, freely mixable, with `#` comments: Combinatory Logic (`S`/`K`/`I` + parens), Unlambda (`` ` `` application), Iota (`*` application, with `i` as the iota combinator when it is a direct operand of `*`), and Jot (`0`/`1`, one number spanning any whitespace and comments). Behaviour matches the reference interpreter across every notation — verified by differential fuzzing (0 mismatches over thousands of programs) and by running its `reverse`, `rot13`, and `hello` example programs.
- Embedding: `zonu_lazyk::run(program_src, input, output)` runs a program against any `Read`/`Write` pair.
- Performance: an ION-style combinator VM with extended combinators (`B`/`C`/`S'`/`B'`/`C'`) introduced by a peephole optimizer, native integer numerals with O(1) church2int extraction, 8-byte packed heap cells, and a Cheney copying collector that bounds memory on unbounded streams. On 20 KB inputs it runs on par with the reference interpreter. Design decisions are recorded in `docs/adr/`.

[Unreleased]: https://github.com/zonuexe/zonu-lazyk/commits/master
