//! Unified parser for the four Lazy K notations (freely mixable, `#` comments).
//!
//!   - Unlambda:            `` ` `` apply; `s` `k` `i`
//!   - Combinatory Logic:   `S` `K` `I`, parens, juxtaposition
//!   - Iota:                `*` apply; `i` iota
//!   - Jot:                 binary `0`/`1`
//!
//! All notations lower to the same core [`Term`] (SKI only). Correctness is
//! pinned against tromp's reference programs.

use crate::term::Term;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    /// Byte offset into the source where parsing failed.
    pub offset: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Parse a Lazy K program in any mixture of the four notations into a core term.
pub fn parse(_src: &str) -> Result<Term, ParseError> {
    // TODO: implement the unified scanner + parser.
    //   1. strip `#`..EOL comments
    //   2. dispatch per token across the four notations, sharing an app stack
    //   3. lower Iota (`i` = \x. x S K) and Jot to SKI
    todo!("parser::parse")
}
