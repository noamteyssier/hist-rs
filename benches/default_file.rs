//! Benchmark `hist` without options on 1M file input.

mod prepare_input;

use std::process::{Command, Stdio};

use assert_cmd::cargo;
use criterion::{Criterion, criterion_group, criterion_main};

pub fn default_opts_benchmark(c: &mut Criterion) {
    let input_data =
        prepare_input::InputData::new(1_000_000).expect("failed to generate input data");
    let path = input_data.path().to_path_buf();
    c.bench_function("default_opts file 1M", |b| {
        b.iter(|| {
            Command::new(cargo::cargo_bin!("hist"))
                .arg(&path)
                .stdout(Stdio::null())
                .status()
                .expect("failed to run hist <file>")
        })
    });

    input_data.close().unwrap_or_else(|err| {
        panic!(
            "failed to clean up input data `{}` due to: {}",
            path.display(),
            err
        )
    });
}

criterion_group!(benches, default_opts_benchmark);
criterion_main!(benches);
