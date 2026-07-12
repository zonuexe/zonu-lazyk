//! CLI: `zonu-lazyk <program-file>` — read the program from a file, feed stdin
//! as the input byte stream, write the output byte stream to stdout.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: zonu-lazyk <program-file>");
        return ExitCode::from(2);
    };

    let src = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("zonu-lazyk: {}: {e}", path.to_string_lossy());
            return ExitCode::FAILURE;
        }
    };

    let stdin = std::io::stdin().lock();
    let stdout = std::io::stdout().lock();

    match zonu_lazyk::run(&src, stdin, stdout) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("zonu-lazyk: {e}");
            ExitCode::FAILURE
        }
    }
}
