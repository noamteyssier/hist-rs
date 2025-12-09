#[path = "../benches/prepare_input.rs"]
mod prepare_input;

use std::{
    ffi::OsStr,
    io::Cursor,
    process::{Command, Stdio},
};

use anyhow::Result;
use assert_cmd::cargo;
use bstr::{ByteSlice, io::BufReadExt};

fn run_file_test<I: IntoIterator>(lines: u64, extra_options: I) -> Result<()>
where
    I::Item: AsRef<OsStr>,
{
    let input_data = prepare_input::InputData::new(lines).expect("failed to generate input data");
    let path = input_data.path().to_path_buf();
    let stdout = Command::new(cargo::cargo_bin!("hist"))
        .args(extra_options)
        .arg(&path)
        .stdout(Stdio::piped())
        .output()
        .expect("failed to run hist <file>")
        .stdout;
    let reader = Cursor::new(stdout);
    let mut line_count = 0;
    for line in reader.byte_lines() {
        let line = line?;
        let (count, token) = line.split_once_str(b"\t").unwrap();
        let count = str::from_utf8(count)?.parse::<usize>()?;
        assert_eq!(input_data.groundtruth.get(token), Some(&count));
        line_count += 1;
    }
    assert_eq!(input_data.groundtruth.len(), line_count);
    input_data.close()?;
    Ok(())
}

#[test]
fn test_default_file() -> Result<()> {
    run_file_test(1_000_000, Vec::<String>::new())
}

#[test]
fn test_multithreading_file() -> Result<()> {
    run_file_test(5_000_000, &["-T", "5"])
}
