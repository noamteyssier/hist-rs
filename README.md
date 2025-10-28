# hist

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.md)
[![Crates.io](https://img.shields.io/crates/d/hist-rs?color=orange&label=crates.io)](https://crates.io/crates/hist-rs)

A high-throughput CLI to count unique lines.

This is a standalone tool with equivalent functionality to `sort | uniq -c | sort -n`.

There is also support for deduplicating an input stream (i.e. only printing unique lines).

## Installation

```bash
cargo install hist-rs
```

## Usage

```bash
# count unique lines in a file
hist <file>

# count unique lines from stdin
/bin/cat <file> | hist

# skip counting and just write unique lines (in the same order as they appear)
hist <file> -u

# exclude lines matching a pattern while counting
hist <file> -e <pattern>

# include lines matching a pattern while counting
hist <file> -i <pattern>

# only output lines with abundance greater than or equal to a threshold
hist <file> -m <threshold>

# only output lines with abundance less than or equal to a threshold
hist <file> -M <threshold>

# sort output by the key (default: by abundance)
hist <file> -n

# sort output in descending order (default: ascending)
hist <file> -d

# show the last-k entries and a count of the other entries (lines + number of elements)
hist <file> -k <k>
```

## Benchmarks

### Benchmarks `(sort | uniq -c | sort -n)`

I use [`nucgen`](https://crates.io/crates/nucgen) to generate a random 1M line [FASTQ file](https://en.wikipedia.org/wiki/FASTQ_format) and pipe it into different tools to compare their throughput with [`hyperfine`](https://lib.rs/crates/hyperfine).

I am measuring the performance of equivalent `sort <file | uniq -c | sort -n` functionality.

Tools compared:
- [`hist`](https://lib.rs/crates/hist-rs)
- [`cuniq`](https://lib.rs/crates/cuniq)
- [`huniq`](https://lib.rs/crates/huniq)
- [`sortuniq`](https://lib.rs/crates/sortuniq)
- [`awk`](https://www.gnu.org/software/gawk/manual/gawk.html)
- Naive Implementation (coreutils `sort <file | uniq -c | sort -n`)
- Naive implementation ([rust-coreutils](https://github.com/uutils/coreutils) `sort <file | uniq -c | sort -n`)
- Naive no cache (LC_ALL=C)
- Naive no cache size hints (LC_ALL=C; size hints for `sort`)

For the specific commands used please check the [`justfile`](./justfile).

#### Benchmark Table `(sort | uniq -c | sort -n)`

> Measured on MacBook M3 Pro

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `hist` | 231.5 ± 4.6 | 226.4 | 243.8 | 1.00 |
| `cuniq` | 561.9 ± 12.4 | 538.2 | 576.9 | 2.43 ± 0.07 |
| `naive-rust` | 890.1 ± 4.5 | 883.6 | 897.5 | 3.84 ± 0.08 |
| `naive-no-locale-size-hints` | 1179.2 ± 40.2 | 1132.8 | 1241.6 | 5.09 ± 0.20 |
| `naive-no-locale` | 1219.5 ± 28.7 | 1188.2 | 1276.6 | 5.27 ± 0.16 |
| `awk` | 1265.4 ± 7.1 | 1254.5 | 1278.5 | 5.47 ± 0.11 |
| `huniq` | 2814.8 ± 67.8 | 2735.9 | 2951.7 | 12.16 ± 0.38 |
| `sortuniq` | 3166.9 ± 71.3 | 3121.1 | 3351.3 | 13.68 ± 0.41 |
| `naive-size-hints` | 5610.1 ± 53.8 | 5542.2 | 5691.7 | 24.23 ± 0.53 |
| `naive` | 5637.6 ± 67.2 | 5527.9 | 5781.1 | 24.35 ± 0.56 |

### Benchmarks (deduplicate stream)

I use [`nucgen`](https://crates.io/crates/nucgen) to generate a random 1M line [FASTQ file](https://en.wikipedia.org/wiki/FASTQ_format) and pipe it into different tools to compare their throughput with [`hyperfine`](https://lib.rs/crates/hyperfine).

I am measuring the performance of deduplicating an input stream (i.e. only printing unique lines).

Tools compared:
- [`hist`](https://lib.rs/crates/hist-rs)
- [`huniq`](https://lib.rs/crates/huniq)
- [`uq`](https://lib.rs/crates/uq)
- [`ripuniq`](https://lib.rs/crates/ripuniq)
- [`unic`](https://github.com/donatj/unic)
- [`awk`](https://www.gnu.org/software/gawk/manual/gawk.html)

For the specific commands used please check the [`justfile`](./justfile).

#### Benchmark Table (deduplicate stream)

> Measured on MacBook M3 Pro

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `hist` | 180.6 ± 17.1 | 171.4 | 245.0 | 1.00 |
| `ripuniq` | 226.1 ± 2.6 | 223.8 | 233.0 | 1.25 ± 0.12 |
| `awk` | 1299.0 ± 10.4 | 1289.3 | 1320.3 | 7.19 ± 0.68 |
| `huniq` | 2418.6 ± 41.5 | 2365.8 | 2488.9 | 13.39 ± 1.29 |
| `uq` | 2990.3 ± 61.9 | 2899.9 | 3120.2 | 16.56 ± 1.61 |
| `unic` | 7947.0 ± 36.0 | 7869.3 | 7979.1 | 44.01 ± 4.17 |
