//! Behavioural conformance tests for the interpreter pipeline.

use std::io::Cursor;
use zonu_lazyk::term::{Comb, Term};
use zonu_lazyk::vm::Vm;

fn run(program: &str, input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    zonu_lazyk::run(program, Cursor::new(input.to_vec()), &mut out).expect("program ran");
    out
}

fn c(k: Comb) -> Term {
    Term::comb(k)
}
fn ap(f: Term, x: Term) -> Term {
    Term::app(f, x)
}

/// Run a directly-constructed term as the program (bypassing parse/optimize).
fn run_term(term: &Term, input: &[u8]) -> Vec<u8> {
    let mut vm = Vm::load(term);
    let mut out = Vec::new();
    vm.run(Cursor::new(input.to_vec()), &mut out)
        .expect("program ran");
    out
}

/// Run a source program with and without the peephole pass.
fn run_both(program: &str, input: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let term = zonu_lazyk::parser::parse(program).expect("parsed");
    let unopt = run_term(&term, input);
    let opt = run_term(&zonu_lazyk::compile::optimize(term), input);
    (opt, unopt)
}

/// `I` is the identity program: output stream == input stream (`cat`).
#[test]
fn identity_is_cat() {
    assert_eq!(run("I", b"hello"), b"hello");
}

/// Empty input to `cat` yields empty output (first head is already EOF).
#[test]
fn cat_empty_input() {
    assert_eq!(run("I", b""), b"");
}

/// All byte values round-trip through `cat`, incl. 0 and 255.
#[test]
fn cat_all_bytes() {
    let bytes: Vec<u8> = (0..=255u8).collect();
    assert_eq!(run("I", &bytes), bytes);
}

/// The same identity program written in every notation must agree.
#[test]
fn identity_across_notations() {
    // I ; unlambda i ; CC-parenthesised ; `SKK` = I ; `` `` `` variants.
    for prog in ["I", "i", "(I)", "SKK", "``skk", "SKS", "(S)(K)(K)"] {
        assert_eq!(
            run(prog, b"Ok"),
            b"Ok",
            "program {prog:?} should be identity"
        );
    }
}

/// Comments and whitespace are ignored.
#[test]
fn comments_and_whitespace() {
    let prog = "# leading comment\n  I  # trailing\n";
    assert_eq!(run(prog, b"xy"), b"xy");
}

/// Every reducer rewrite rule has an identity witness `≡ I`; running it as the
/// program must therefore behave as `cat`. This exercises S/K/I, B, C, and the
/// balanced combinators S'/B'/C', plus native `Num`/`Inc` extraction.
#[test]
fn every_combinator_rule_via_identity_witness() {
    use Comb::*;
    let witnesses = [
        ("I", c(I)),
        ("S K K", ap(ap(c(S), c(K)), c(K))),
        ("B I I", ap(ap(c(B), c(I)), c(I))),
        ("C K I", ap(ap(c(C), c(K)), c(I))),
        ("S' K I I", ap(ap(ap(c(Sp), c(K)), c(I)), c(I))),
        ("B' I I I", ap(ap(ap(c(Bp), c(I)), c(I)), c(I))),
        ("C' K I I", ap(ap(ap(c(Cp), c(K)), c(I)), c(I))),
    ];
    for (name, term) in &witnesses {
        assert_eq!(
            run_term(term, b"Zz!\x00\xff"),
            b"Zz!\x00\xff",
            "witness {name}"
        );
    }
}

/// The peephole pass must not change observable behaviour. Each program here is
/// extensionally `I` but written to trigger a specific rewrite.
#[test]
fn peephole_preserves_behaviour() {
    let programs = [
        "S(KI)I",         // S (K p) I  -> p
        "S(KI)(SKK)",     // S (K p) q  -> B p q
        "SK(KI)",         // S p (K q)  -> C p q
        "S(S(KK)(SKK))I", // ... -> S' (balanced)
    ];
    for prog in programs {
        let (opt, unopt) = run_both(prog, b"cat me");
        assert_eq!(opt, unopt, "opt vs unopt for {prog}");
        assert_eq!(opt, b"cat me", "identity behaviour for {prog}");
    }
}
