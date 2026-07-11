//! Throughput benchmarks. `cat`/`reverse`/`rot13` over sized inputs measure the
//! reduction + GC + I/O hot path; the peephole pair quantifies ADR-0003.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::io;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).unwrap()
}

/// Run a source program to completion, discarding output.
fn run(src: &str, input: &[u8]) {
    zonu_lazyk::run(src, io::Cursor::new(input.to_vec()), io::sink()).unwrap();
}

/// Run a pre-parsed term (optionally optimized) to completion.
fn run_term(term: &zonu_lazyk::term::Term, input: &[u8]) {
    let mut vm = zonu_lazyk::vm::Vm::load(term);
    vm.run(io::Cursor::new(input.to_vec()), io::sink()).unwrap();
}

fn bench(c: &mut Criterion) {
    let data: Vec<u8> = (0..2_000u32).map(|i| (i % 128) as u8 + 1).collect();

    let mut io_group = c.benchmark_group("programs/2000B");
    io_group.throughput(criterion::Throughput::Bytes(data.len() as u64));
    io_group.bench_function("cat", |b| b.iter(|| run("I", black_box(&data))));
    let reverse = fixture("reverse.lazy");
    io_group.bench_function("reverse", |b| b.iter(|| run(&reverse, black_box(&data))));
    let rot13 = fixture("rot13.lazy");
    io_group.bench_function("rot13", |b| b.iter(|| run(&rot13, black_box(&data))));
    io_group.finish();

    // Peephole on vs. off, same program (ADR-0003).
    let raw = zonu_lazyk::parser::parse(&reverse).unwrap();
    let opt = zonu_lazyk::compile::optimize(raw.clone());
    let small = &data[..256];
    let mut opt_group = c.benchmark_group("peephole/reverse-256B");
    opt_group.bench_function("unoptimized", |b| {
        b.iter(|| run_term(&raw, black_box(small)))
    });
    opt_group.bench_function("optimized", |b| b.iter(|| run_term(&opt, black_box(small))));
    opt_group.finish();

    c.bench_function("parse+optimize/reverse.lazy", |b| {
        b.iter(|| {
            let t = zonu_lazyk::parser::parse(black_box(&reverse)).unwrap();
            black_box(zonu_lazyk::compile::optimize(t));
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
