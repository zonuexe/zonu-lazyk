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

Reference-compatible across all four notations (CC, Unlambda, Iota with the
`ι` combinator under `*`, and Jot). The full pipeline runs — parser, peephole
optimizer, ION-style reducer with extended combinators and native numerals, and
a Cheney copying GC that bounds the heap on streaming workloads.

Verified by:

- **Conformance fixtures** from the reference distribution — `reverse.lazy`
  (Jot), `rot13.lazy` (CC/Unlambda), `hello.lazy` (mixed) — in `tests/`.
- **Differential fuzzing** against the reference interpreter: 0 mismatches over
  thousands of random programs, including pure-iota and whitespace-split Jot.
- Unit tests for every reducer and peephole rule, notation equivalence, and GC
  under pressure.

`cargo bench` has criterion throughput benchmarks. church2int extraction is
O(1) per numeral; `cat` runs at ~18 MiB/s. Programs bound by their own reduction
(e.g. `rot13`, `reverse`) are the next optimization target.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
