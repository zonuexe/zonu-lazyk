//! Reference-program conformance tests (ADR-0001..0004 correctness gate).
//!
//! Port tromp's example programs and assert exact output. Enable each as the
//! corresponding pipeline stage lands.

use std::io::Cursor;

/// `I` is the identity program: output stream == input stream (`cat`).
#[test]
#[ignore = "pipeline not implemented yet"]
fn identity_is_cat() {
    let mut out = Vec::new();
    zonu_lazyk::run("I", Cursor::new(b"hello".to_vec()), &mut out).unwrap();
    assert_eq!(out, b"hello");
}

// TODO: reverse, hello-world, primes, ackermann, and the four-notation
// equivalence tests (the same program in Unlambda/CC/Iota/Jot must agree).
