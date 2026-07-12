//! zonu-lazyk — a performance-oriented Lazy K interpreter, embeddable as a
//! library.
//!
//! A Lazy K program is a pure function from an input byte stream to an output
//! byte stream. Compile it once and run it against any `Read`/`Write` pair:
//!
//! ```
//! let prog = zonu_lazyk::Program::compile("I").unwrap(); // `I` is `cat`
//! let out = prog.eval(b"hello").unwrap();
//! assert_eq!(out, b"hello");
//! ```
//!
//! For untrusted programs (Lazy K can loop forever or emit an unbounded stream),
//! set [`Limits`]:
//!
//! ```
//! use zonu_lazyk::{Limits, Program};
//! let prog = Program::compile("``SII``SII").unwrap(); // diverges
//! let limits = Limits { max_steps: Some(100_000), ..Limits::none() };
//! assert!(matches!(prog.eval_with(b"", &limits), Err(zonu_lazyk::Error::StepLimit)));
//! ```
//!
//! **Stability:** while zonu-lazyk is `0.x`, `Program`/`Limits`/`Error` may
//! change between minor versions — pin an exact version. The pipeline modules
//! ([`parser`], [`compile`], [`vm`], [`io`], [`term`]) are exposed only as
//! unstable internals (they back the CLI, tests, and benches), not the embedding
//! contract. See `docs/adr/0006-embedding-api.md` and `-0007-embedding-hardening.md`.

#[doc(hidden)]
pub mod compile;
#[doc(hidden)]
pub mod io;
#[doc(hidden)]
pub mod parser;
#[doc(hidden)]
pub mod term;
#[doc(hidden)]
pub mod vm;

pub use parser::ParseError;

/// A compiled Lazy K program: parsed and optimized once, run many times.
#[derive(Clone, Debug)]
pub struct Program {
    term: term::Term,
}

impl Program {
    /// Parse and optimize a program written in any of the four notations.
    pub fn compile(source: &str) -> Result<Program, ParseError> {
        Ok(Program {
            term: compile::optimize(parser::parse(source)?),
        })
    }

    /// Run over `input`, writing the output byte stream to `output`. Unlimited.
    pub fn run<R: std::io::Read + 'static, W: std::io::Write>(
        &self,
        input: R,
        output: W,
    ) -> Result<(), Error> {
        self.run_with(input, output, &Limits::none())
    }

    /// Run with resource limits (ADR-0007).
    pub fn run_with<R: std::io::Read + 'static, W: std::io::Write>(
        &self,
        input: R,
        output: W,
        limits: &Limits,
    ) -> Result<(), Error> {
        let mut vm = vm::Vm::load(&self.term);
        vm.set_limits(limits);
        vm.run(input, output)
    }

    /// Run over `input` bytes and collect the output. Unlimited.
    pub fn eval(&self, input: &[u8]) -> Result<Vec<u8>, Error> {
        self.eval_with(input, &Limits::none())
    }

    /// Run over `input` bytes with resource limits, collecting the output.
    pub fn eval_with(&self, input: &[u8], limits: &Limits) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        self.run_with(std::io::Cursor::new(input.to_vec()), &mut out, limits)?;
        Ok(out)
    }
}

/// Opt-in resource limits for one run. `None` means unlimited; [`Limits::none`]
/// (the `Default`) sets no limits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Limits {
    /// Maximum reduction steps before a run stops with [`Error::StepLimit`].
    /// Bounds a program that never produces the next output byte.
    pub max_steps: Option<u64>,
    /// Maximum output bytes before a run stops with [`Error::OutputLimit`].
    /// Bounds a program with well-formed but unbounded output.
    pub max_output_bytes: Option<u64>,
}

impl Limits {
    /// No limits (the same as `Limits::default()`).
    pub const fn none() -> Limits {
        Limits {
            max_steps: None,
            max_output_bytes: None,
        }
    }
}

/// Parse and run `program_src` over `input`, writing output to `output`. A
/// one-shot convenience; use [`Program`] to run a program more than once.
pub fn run<R: std::io::Read + 'static, W: std::io::Write>(
    program_src: &str,
    input: R,
    output: W,
) -> Result<(), Error> {
    Program::compile(program_src)?.run(input, output)
}

/// An error from compiling or running a program.
#[derive(Debug)]
pub enum Error {
    /// The program did not parse.
    Parse(ParseError),
    /// The program's output was not a well-formed stream of numerals.
    IllFormedOutput(String),
    /// A host I/O error while reading input or writing output.
    Io(std::io::Error),
    /// The `max_steps` limit was reached (ADR-0007).
    StepLimit,
    /// The `max_output_bytes` limit was reached (ADR-0007).
    OutputLimit,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "{e}"),
            Error::IllFormedOutput(msg) => write!(f, "ill-formed output: {msg}"),
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::StepLimit => write!(f, "reduction-step limit reached"),
            Error::OutputLimit => write!(f, "output-byte limit reached"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}
