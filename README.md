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

## Embedding

zonu-lazyk is also a library. Compile a program once and run it against any
`Read`/`Write` pair or in-memory bytes:

```rust
use zonu_lazyk::Program;

let cat = Program::compile("I")?;          // `I` is the identity — Lazy K's `cat`
let out = cat.eval(b"hello")?;             // -> Vec<u8>
assert_eq!(out, b"hello");
```

For untrusted programs (Lazy K can loop forever or emit an unbounded stream), set
[`Limits`]:

```rust
use zonu_lazyk::{Error, Limits, Program};

let omega = Program::compile("``SII``SII")?;    // diverges
let limits = Limits { max_steps: Some(100_000), ..Limits::none() };
assert!(matches!(omega.eval_with(b"", &limits), Err(Error::StepLimit)));
```

To decode a *value* rather than a byte stream, evaluate a term to a Church
numeral or read the output list as raw numerals:

```rust
use zonu_lazyk::{DecodeOptions, Program, church_numeral};

// A computed number, with no 256 cap and no byte-counting.
assert_eq!(Program::from_term(church_numeral(1000)).eval_numeral()?, 1000);

// The output list as raw Church numerals instead of bytes.
let vals = Program::compile("I")?.eval_values(b"Hi", &DecodeOptions::default())?;
assert_eq!(vals, vec![72, 105]);
```

See [`examples/embed.rs`](examples/embed.rs). The API is `0.x`-unstable — pin an
exact version. The pipeline modules are exposed only as unstable internals.

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

`cargo bench` has criterion throughput benchmarks. Heap cells are packed to 8
bytes and church2int extraction is O(1) per numeral; on 20 KB inputs the
interpreter runs on par with the reference (`rot13` matches it, `reverse` is
within ~1.1x). See [ADR-0005](docs/adr/0005-performance-findings.md) for the
profiling and roadmap.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
