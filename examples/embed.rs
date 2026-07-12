//! Embedding zonu-lazyk in a Rust program.
//!
//!   cargo run --example embed

use zonu_lazyk::{Error, Limits, Program};

fn main() {
    // Compile a program once (here `I`, the identity — Lazy K's `cat`).
    let cat = Program::compile("I").expect("valid program");

    // Run it against in-memory bytes.
    let out = cat.eval(b"Hello, embedding!").expect("runs");
    println!("cat: {}", String::from_utf8_lossy(&out));

    // A program from the reference test suite: rot13 (Combinatory Logic).
    // (Any of the four notations works; compile from a &str.)
    // Here we just show streaming to a Writer instead of collecting a Vec.
    let mut buf = Vec::new();
    cat.run(std::io::Cursor::new(b"streamed".to_vec()), &mut buf)
        .expect("runs");
    println!("streamed: {}", String::from_utf8_lossy(&buf));

    // Bound an untrusted program: this one diverges, so cap the reduction steps.
    let omega = Program::compile("``SII``SII").expect("valid program");
    let limits = Limits {
        max_steps: Some(100_000),
        ..Limits::none()
    };
    match omega.eval_with(b"", &limits) {
        Err(Error::StepLimit) => println!("omega: stopped at the step limit (as expected)"),
        other => println!("omega: unexpected {other:?}"),
    }
}
