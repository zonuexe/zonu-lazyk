//! Decode helpers for embedders (ADR-0008): numerals, value lists, term input.

use zonu_lazyk::{Comb, DecodeOptions, Error, Limits, Program, Term, church_numeral};

/// `eval_numeral` reads a Church numeral directly — including values > 255,
/// with no byte-counting workaround.
#[test]
fn eval_numeral_reads_values_beyond_255() {
    for n in [0u32, 1, 42, 255, 256, 1000, 70_000] {
        let prog = Program::from_term(church_numeral(n));
        assert_eq!(prog.eval_numeral().unwrap(), n as u64, "numeral {n}");
    }
}

/// A numeral built from the AST (Term) round-trips — `from_term` avoids the
/// render-to-string / re-parse trip.
#[test]
fn eval_numeral_from_built_term() {
    let s = || Term::comb(Comb::S);
    let k = || Term::comb(Comb::K);
    // succ = S (S (K S) K); succ n f x = f (n f x).
    let succ = || Term::app(s(), Term::app(Term::app(s(), Term::app(k(), s())), k()));
    let two = Term::app(succ(), Term::app(succ(), church_numeral(0)));
    assert_eq!(Program::from_term(two).eval_numeral().unwrap(), 2);
}

/// `eval_values` reads the output list as raw numerals, not bytes. `cat` echoes
/// its input, so the values are the input bytes.
#[test]
fn eval_values_reads_the_list() {
    let cat = Program::compile("I").unwrap();
    let opts = DecodeOptions::default(); // eof = Some(256)
    assert_eq!(cat.eval_values(b"Hi!", &opts).unwrap(), vec![72, 105, 33]);
    assert_eq!(cat.eval_values(b"", &opts).unwrap(), Vec::<u64>::new());
}

/// With the EOF sentinel disabled, `max_values` takes a bounded prefix — here of
/// the infinite trailing-256 input stream `cat` echoes past EOF.
#[test]
fn eval_values_take_prefix_without_eof() {
    let cat = Program::compile("I").unwrap();
    let opts = DecodeOptions {
        eof: None,
        max_values: Some(4),
        max_steps: None,
    };
    // 'a', 'b', then EOF 256, 256 (the stream continues with 256s).
    assert_eq!(
        cat.eval_values(b"ab", &opts).unwrap(),
        vec![97, 98, 256, 256]
    );
}

/// The step limit still guards divergence in numeral decoding.
#[test]
fn eval_numeral_respects_step_limit() {
    let omega = Program::compile("``SII``SII").unwrap();
    let limits = Limits {
        max_steps: Some(100_000),
        ..Limits::none()
    };
    assert!(matches!(
        omega.eval_numeral_with(&limits),
        Err(Error::StepLimit)
    ));
}
