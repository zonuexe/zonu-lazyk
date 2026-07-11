# zonu-lazyk

A performance-oriented [Lazy K](https://tromp.github.io/cl/lazy-k.html) interpreter in Rust.

Lazy K is a pure, lazy, functional language whose only primitives are the SKI
combinators. A program is a single combinator term that maps an input byte
stream to an output byte stream.

## Usage

```sh
zonu-lazyk <program-file> < input > output
```

The program is read from `<program-file>` in any mixture of the four notations
(Unlambda, Combinatory Logic, Iota, Jot). Standard input is the input byte
stream; standard output receives the output byte stream. A numeral `>= 256`
signals end of stream.

## Design

The interpreter compiles the program and reduces it on a custom combinator VM.
Key decisions are recorded as ADRs:

- [ADR-0001](docs/adr/0001-ion-style-combinator-bytecode-vm.md) — ION-style combinator bytecode VM
- [ADR-0002](docs/adr/0002-flat-array-heap-with-cheney-gc.md) — flat-array heap + Cheney copying GC
- [ADR-0003](docs/adr/0003-extended-combinators-peephole.md) — peephole rewrite into extended combinators
- [ADR-0004](docs/adr/0004-native-integer-church-numerals.md) — native integer Church numerals at the I/O boundary

Domain vocabulary lives in [CONTEXT.md](CONTEXT.md).

## Status

Scaffold. Module layout is in place with the pipeline stubbed; see `todo!()`
markers in `src/` and the ignored tests in `tests/reference.rs`.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
