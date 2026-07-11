//! Unified parser for the four Lazy K notations (freely mixable, `#` comments).
//!
//!   - Unlambda:            `` ` `` apply (prefix); `s` `k` `i`
//!   - Combinatory Logic:   `S` `K` `I`, parens, juxtaposition (left-assoc)
//!   - Iota:                `*` apply (prefix)
//!   - Jot:                 binary `0`/`1` runs
//!
//! Following the reference interpreter, lowercase `i` is the `I` combinator (not
//! Barker's iota combinator); `*` and `` ` `` are both prefix application. All
//! notations lower to the same core [`Term`] (SKI only).

use crate::term::{Comb, Term};

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
pub fn parse(src: &str) -> Result<Term, ParseError> {
    let mut p = Parser {
        bytes: src.as_bytes(),
        pos: 0,
    };
    let term = p.parse_sequence(false)?;
    p.skip_trivia();
    if p.pos != p.bytes.len() {
        return Err(p.error("unexpected trailing input"));
    }
    Ok(term)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            offset: self.pos,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek();
        if b.is_some() {
            self.pos += 1;
        }
        b
    }

    /// Skip whitespace and `#`..end-of-line comments.
    fn skip_trivia(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b' ' | b'\t' | b'\r' | b'\n' | 0x0c => self.pos += 1,
                b'#' => {
                    while let Some(c) = self.peek() {
                        self.pos += 1;
                        if c == b'\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    /// Parse a left-associative juxtaposition of terms. Stops at end of input,
    /// or at `)` when `in_parens` is set (the `)` is left for the caller).
    fn parse_sequence(&mut self, in_parens: bool) -> Result<Term, ParseError> {
        let mut acc: Option<Term> = None;
        loop {
            self.skip_trivia();
            match self.peek() {
                None => break,
                Some(b')') => {
                    if in_parens {
                        break;
                    }
                    return Err(self.error("unmatched `)`"));
                }
                Some(_) => {
                    let t = self.parse_term()?;
                    acc = Some(match acc {
                        None => t,
                        Some(f) => Term::app(f, t),
                    });
                }
            }
        }
        acc.ok_or_else(|| self.error("empty expression"))
    }

    /// Parse a single term: an atom, a prefix application, a parenthesized
    /// sequence, or a Jot run.
    fn parse_term(&mut self) -> Result<Term, ParseError> {
        self.skip_trivia();
        let start = self.pos;
        match self.bump() {
            None => Err(ParseError {
                message: "unexpected end of input".to_string(),
                offset: start,
            }),
            Some(b'`') | Some(b'*') => {
                let f = self.parse_term()?;
                let x = self.parse_term()?;
                Ok(Term::app(f, x))
            }
            Some(b'(') => {
                let inner = self.parse_sequence(true)?;
                self.skip_trivia();
                if self.bump() != Some(b')') {
                    return Err(ParseError {
                        message: "expected `)`".to_string(),
                        offset: self.pos,
                    });
                }
                Ok(inner)
            }
            Some(b's') | Some(b'S') => Ok(Term::comb(Comb::S)),
            Some(b'k') | Some(b'K') => Ok(Term::comb(Comb::K)),
            Some(b'i') | Some(b'I') => Ok(Term::comb(Comb::I)),
            Some(b'0') | Some(b'1') => {
                self.pos = start; // let the jot reader consume the whole run
                Ok(self.parse_jot())
            }
            Some(other) => Err(ParseError {
                message: format!("unexpected character {:?}", other as char),
                offset: start,
            }),
        }
    }

    /// Parse a maximal run of `0`/`1` as a Jot program.
    ///
    ///   [ε]  = I
    ///   [w0] = [w] S K
    ///   [w1] = S (K [w])
    fn parse_jot(&mut self) -> Term {
        let mut acc = Term::comb(Comb::I);
        while let Some(bit @ (b'0' | b'1')) = self.peek() {
            self.pos += 1;
            acc = if bit == b'0' {
                Term::app(Term::app(acc, Term::comb(Comb::S)), Term::comb(Comb::K))
            } else {
                Term::app(Term::comb(Comb::S), Term::app(Term::comb(Comb::K), acc))
            };
        }
        acc
    }
}
