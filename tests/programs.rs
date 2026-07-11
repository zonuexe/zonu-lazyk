//! End-to-end conformance against real Lazy K programs from the reference
//! distribution (see `tests/fixtures/NOTICE.md`). Each exercises a different mix
//! of notations, so together they pin the parser, reducer, GC, and I/O against
//! known behaviour.

use std::io::Cursor;

fn run_file(name: &str, input: &[u8]) -> Vec<u8> {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(&path).expect("read fixture");
    let mut out = Vec::new();
    zonu_lazyk::run(&src, Cursor::new(input.to_vec()), &mut out).expect("program ran");
    out
}

fn rot13(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .map(|&b| match b {
            b'a'..=b'z' => (b - b'a' + 13) % 26 + b'a',
            b'A'..=b'Z' => (b - b'A' + 13) % 26 + b'A',
            other => other,
        })
        .collect()
}

/// `hello.lazy` (CC + Unlambda + Jot digits) ignores input and prints a string.
#[test]
fn hello_world() {
    assert_eq!(run_file("hello.lazy", b""), b"Hello, world!");
    // Input is ignored.
    assert_eq!(run_file("hello.lazy", b"whatever"), b"Hello, world!");
}

/// `reverse.lazy` is a single Jot number spanning many lines — it pins the
/// whitespace-spanning Jot tokenization. It reverses the input byte stream.
#[test]
fn reverse() {
    assert_eq!(run_file("reverse.lazy", b"abc"), b"cba");
    assert_eq!(
        run_file("reverse.lazy", b"Hello, Lazy K!"),
        b"!K yzaL ,olleH"
    );
    assert_eq!(run_file("reverse.lazy", b""), b"");
    let bytes: Vec<u8> = (0..64u8).collect();
    let mut rev = bytes.clone();
    rev.reverse();
    assert_eq!(run_file("reverse.lazy", &bytes), rev);
}

/// `rot13.lazy` (CC/Unlambda) transforms each input byte — exercises higher-order
/// use of input numerals, which pins the argument-swap `Inc` protocol.
#[test]
fn rot13_roundtrip() {
    let msg = b"The Quick Brown Fox! 123";
    assert_eq!(run_file("rot13.lazy", msg), rot13(msg));
    // rot13 is its own inverse.
    assert_eq!(run_file("rot13.lazy", &rot13(msg)), msg);
}
