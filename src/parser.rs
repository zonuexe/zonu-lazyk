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

/// Barker's iota combinator `ι = \f. f S K`, as the SKI term
/// `S (S I (K S)) (K K)` (the reference's encoding).
fn iota() -> Term {
    let s = || Term::comb(Comb::S);
    let k = || Term::comb(Comb::K);
    let i = || Term::comb(Comb::I);
    // S (S I (K S)) (K K)
    let si_ks = Term::app(Term::app(s(), i()), Term::app(k(), s()));
    Term::app(Term::app(s(), si_ks), Term::app(k(), k()))
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
                    let t = self.parse_term(false)?;
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
    ///
    /// `i_is_iota` is set only while parsing a direct operand of `*`: there a
    /// lowercase `i` denotes Barker's iota combinator, matching the reference
    /// (`parse_expr(..., ch == '*')`). Everywhere else `i` is `I`.
    fn parse_term(&mut self, i_is_iota: bool) -> Result<Term, ParseError> {
        self.skip_trivia();
        let start = self.pos;
        match self.bump() {
            None => Err(ParseError {
                message: "unexpected end of input".to_string(),
                offset: start,
            }),
            Some(op @ (b'`' | b'*')) => {
                let operand_is_iota = op == b'*';
                let f = self.parse_term(operand_is_iota)?;
                let x = self.parse_term(operand_is_iota)?;
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
            // Lowercase `i` is the iota combinator only as a direct `*` operand.
            Some(b'i') if i_is_iota => Ok(iota()),
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

    /// Parse a Jot program: a run of `0`/`1` bits.
    ///
    ///   [ε]  = I
    ///   [w0] = [w] S K
    ///   [w1] = S (K [w])
    ///
    /// The reference reader skips whitespace and comments at the character
    /// level everywhere (`getch` loops `while (isspace(ch))`), so a single Jot
    /// number continues across whitespace and `#` comments — the bits need not
    /// be contiguous.
    fn parse_jot(&mut self) -> Term {
        let mut acc = Term::comb(Comb::I);
        loop {
            self.skip_trivia();
            match self.peek() {
                Some(b'0') => {
                    self.pos += 1;
                    acc = Term::app(Term::app(acc, Term::comb(Comb::S)), Term::comb(Comb::K));
                }
                Some(b'1') => {
                    self.pos += 1;
                    acc = Term::app(Term::comb(Comb::S), Term::app(Term::comb(Comb::K), acc));
                }
                _ => break,
            }
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::{iota, parse};
    use crate::term::{Comb, Term};

    fn i() -> Term {
        Term::comb(Comb::I)
    }

    #[test]
    fn lowercase_i_is_iota_only_as_star_operand() {
        // `i` is the iota combinator only as a direct operand of `*`.
        assert_eq!(parse("*ii").unwrap(), Term::app(iota(), iota()));
        // Under backtick, at top level, and uppercase `I` are all `I`.
        assert_eq!(parse("`ii").unwrap(), Term::app(i(), i()));
        assert_eq!(parse("ii").unwrap(), Term::app(i(), i()));
        assert_eq!(parse("*II").unwrap(), Term::app(i(), i()));
        // Nested: only the immediate `*` operand `i` is iota; inside parens it is I.
        assert_eq!(parse("*(i)i").unwrap(), Term::app(i(), iota()));
    }

    #[test]
    fn jot_spans_whitespace_and_comments() {
        // One Jot number split by whitespace/newlines/comments == contiguous.
        let contiguous = parse("10110").unwrap();
        assert_eq!(parse("1 0 1 1 0").unwrap(), contiguous);
        assert_eq!(parse("10\n11\n0").unwrap(), contiguous);
        assert_eq!(parse("101 # c\n10").unwrap(), contiguous);
    }
}
