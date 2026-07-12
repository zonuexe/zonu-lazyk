//! The embedding API: `Program`, `Limits`, and `Error` (ADR-0006, ADR-0007).

use zonu_lazyk::{Error, Limits, Program};

/// Compile once, run many times with different inputs.
#[test]
fn program_compiles_once_runs_many() {
    let cat = Program::compile("I").unwrap();
    assert_eq!(cat.eval(b"first").unwrap(), b"first");
    assert_eq!(cat.eval(b"second").unwrap(), b"second");
    // `run` streams to any Write.
    let mut out = Vec::new();
    cat.run(std::io::Cursor::new(b"third".to_vec()), &mut out)
        .unwrap();
    assert_eq!(out, b"third");
}

/// A parse error is reported, not panicked.
#[test]
fn compile_reports_parse_errors() {
    assert!(Program::compile("(").is_err());
}

/// The step limit stops a non-terminating program.
#[test]
fn step_limit_stops_divergence() {
    // S I I (S I I) = ω ω — reduces forever without producing output.
    let omega = Program::compile("``SII``SII").unwrap();
    let limits = Limits {
        max_steps: Some(200_000),
        ..Limits::none()
    };
    assert!(matches!(
        omega.eval_with(b"", &limits),
        Err(Error::StepLimit)
    ));
}

/// The output-byte limit stops an over-long output.
#[test]
fn output_limit_caps_output() {
    let cat = Program::compile("I").unwrap();
    let limits = Limits {
        max_output_bytes: Some(3),
        ..Limits::none()
    };
    // `cat` wants to echo 5 bytes; the limit stops it at 3.
    assert!(matches!(
        cat.eval_with(b"hello", &limits),
        Err(Error::OutputLimit)
    ));
}

/// Generous limits do not disturb a normal run.
#[test]
fn limits_do_not_affect_a_bounded_run() {
    let cat = Program::compile("I").unwrap();
    let limits = Limits {
        max_steps: Some(10_000_000),
        max_output_bytes: Some(1_000),
    };
    assert_eq!(cat.eval_with(b"ok", &limits).unwrap(), b"ok");
}

/// `Error` is a real `std::error::Error` (Display + source).
#[test]
fn error_is_std_error() {
    let err: Error = Program::compile("(").unwrap_err().into();
    let _boxed: Box<dyn std::error::Error> = Box::new(err);
    assert!(!_boxed.to_string().is_empty());
}
