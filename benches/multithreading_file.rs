//! Benchmark `hist` with option `-T` on 10M file input.

mod prepare_input;

use std::process::{Command, Stdio};
use std::time::Duration;

use assert_cmd::cargo;
use criterion::{Criterion, criterion_group, criterion_main};

pub fn multithreading_file_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("multithreading file");
    group.measurement_time(Duration::from_secs(60));
    group.warm_up_time(Duration::from_secs(15));
    let input_data =
        prepare_input::InputData::new(10_000_000).expect("failed to generate input data");
    let path = input_data.path().to_path_buf();
    group.bench_function("single-threaded 10M", |b| {
        b.iter(|| {
            Command::new(cargo::cargo_bin!("hist"))
                .arg(&path)
                .stdout(Stdio::null())
                .status()
                .expect("failed to run hist <file>")
        })
    });
    group.bench_function("5-threaded", |b| {
        b.iter(|| {
            Command::new(cargo::cargo_bin!("hist"))
                .args(&["-T", "5"])
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
    group.finish();
}

criterion_group!(benches, multithreading_file_benchmark);
criterion_main!(benches);
