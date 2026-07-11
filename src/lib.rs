//! zonu-lazyk — a performance-oriented Lazy K interpreter.
//!
//! Pipeline (see `docs/adr/`):
//!   source bytes
//!     -> [`parser`]   : any of the four notations -> core [`term::Term`]
//!     -> [`compile`]  : peephole rewrite into extended combinators (ADR-0003),
//!                       then load into the VM heap
//!     -> [`vm`]       : ION-style combinator reduction on a flat-array heap
//!                       with a Cheney copying GC (ADR-0001, ADR-0002)
//!   driven by [`io`]  : byte-stream <-> Church numerals, native ints (ADR-0004)

pub mod compile;
pub mod io;
pub mod parser;
pub mod term;
pub mod vm;

/// Top-level error type for the whole pipeline.
#[derive(Debug)]
pub enum Error {
    Parse(parser::ParseError),
    /// The program's output was not a well-formed stream of numerals.
    IllFormedOutput(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<parser::ParseError> for Error {
    fn from(e: parser::ParseError) -> Self {
        Error::Parse(e)
    }
}

/// Parse `program_src`, run it over `input`, and write the output byte stream to `output`.
pub fn run<R: std::io::Read, W: std::io::Write>(
    program_src: &str,
    input: R,
    output: W,
) -> Result<(), Error> {
    let term = parser::parse(program_src)?;
    let term = compile::optimize(term);
    let mut vm = vm::Vm::load(&term);
    vm.run(input, output)?;
    Ok(())
}
