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

/// Run a directly-constructed term with an aggressive GC threshold.
fn run_term_gc(term: &Term, input: &[u8], threshold: usize) -> Vec<u8> {
    let mut vm = Vm::load(term);
    vm.set_gc_threshold(threshold);
    let mut out = Vec::new();
    vm.run(Cursor::new(input.to_vec()), &mut out)
        .expect("program ran");
    out
}

/// With the heap collected on almost every step, results must be unchanged —
/// this catches root/relocation bugs in the copying collector.
#[test]
fn gc_under_pressure_is_transparent() {
    use Comb::*;
    let witnesses = [
        c(I),
        ap(ap(c(S), c(K)), c(K)),            // S K K
        ap(ap(c(C), c(K)), c(I)),            // C K I
        ap(ap(ap(c(Sp), c(K)), c(I)), c(I)), // S' K I I
        ap(ap(ap(c(Bp), c(I)), c(I)), c(I)), // B' I I I
    ];
    let input = b"collect me!\x00\x7f\xff";
    for w in &witnesses {
        assert_eq!(run_term_gc(w, input, 64), input);
    }
}

/// A long `cat` run reclaims the heap repeatedly and still echoes exactly.
#[test]
fn gc_cat_large_input() {
    let data: Vec<u8> = (0..20_000u32).map(|i| (i % 256) as u8).collect();
    assert_eq!(run_term_gc(&c(Comb::I), &data, 2048), data);
}

/// Render a term in each surface notation, so one term can be run four ways.
fn render(t: &Term, notation: char) -> String {
    fn go(t: &Term, n: char, out: &mut String) {
        match t {
            // Uppercase atoms are `S`/`K`/`I` in every notation; lowercase `i`
            // would be the iota combinator under `*`, so avoid it here.
            Term::Comb(Comb::S) => out.push('S'),
            Term::Comb(Comb::K) => out.push('K'),
            Term::Comb(Comb::I) => out.push('I'),
            Term::Comb(other) => panic!("cannot render {other:?} (compiler-only)"),
            Term::Num(_) => panic!("cannot render Num"),
            Term::App(a, b) => match n {
                // CC: parenthesized juxtaposition.
                'c' => {
                    out.push('(');
                    go(a, n, out);
                    go(b, n, out);
                    out.push(')');
                }
                // Unlambda: backtick prefix application.
                'u' => {
                    out.push('`');
                    go(a, n, out);
                    go(b, n, out);
                }
                // Iota: star prefix application.
                '*' => {
                    out.push('*');
                    go(a, n, out);
                    go(b, n, out);
                }
                _ => unreachable!(),
            },
        }
    }
    let mut s = String::new();
    go(t, notation, &mut s);
    s
}

/// The same SKI term written in CC, Unlambda, and Iota (`*`) notation must parse
/// and run identically — the core Iota coverage (`i` = I, `*` = apply).
#[test]
fn notation_equivalence() {
    use Comb::*;
    let terms = [
        c(I),
        ap(ap(c(S), c(K)), c(K)),                               // S K K
        ap(ap(c(S), ap(c(K), c(I))), c(I)),                     // S (K I) I
        ap(ap(c(S), c(K)), ap(c(K), c(I))),                     // S K (K I)
        ap(ap(c(S), ap(c(K), c(I))), ap(ap(c(S), c(K)), c(K))), // S (K I)(S K K)
    ];
    for t in &terms {
        let base = run_term(t, b"iOtA?");
        for notation in ['c', 'u', '*'] {
            let src = render(t, notation);
            assert_eq!(run(&src, b"iOtA?"), base, "notation {notation}: {src}");
        }
    }
}

/// In Iota, `*ii` = ι ι = I, so it behaves as `cat`. Exercises `i` = ι under `*`.
#[test]
fn iota_combinator_runs() {
    assert_eq!(run("*ii", b"iota!"), b"iota!");
    assert_eq!(run("* i i", b"OK"), b"OK"); // whitespace between iota tokens
}

/// Regression: whitespace and `#` comments never break a Jot number — the bits
/// of one Jot program may be split across spaces, newlines, and comments (the
/// reference reader skips whitespace at the character level).
#[test]
fn jot_spans_whitespace() {
    // Same Jot number, contiguous vs. split three ways — all must agree.
    let contiguous = run("11100100", b"Zz");
    assert_eq!(run("1 1 1 0 0 1 0 0", b"Zz"), contiguous);
    assert_eq!(run("111\n001\n00", b"Zz"), contiguous);
    assert_eq!(run("11100 # mid\n100", b"Zz"), contiguous);
}

/// Regression: higher-order use of `Inc` (argument-swap protocol). This minimal
/// program applies an input char in a higher-order position; the naive
/// force-to-a-number `Inc` got stuck here.
#[test]
fn higher_order_inc_regression() {
    // ``SS`K`SI on input "ab" yields byte 0x01 (verified against the reference).
    assert_eq!(run("``SS`K`SI", b"ab"), b"\x01");
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
