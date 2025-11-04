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

# create histogram but skip sorting by value
hist <file> -S

# perform pattern substitution on incoming lines (regex compatible pattern matching)
hist <file> -s <pattern> <replacement>

# perform multiple pattern substitutions on incoming lines (can be used multiple times)
hist <file> -s <pattern> <replacement> -s <pattern> <replacement>
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
- [`runiq`](https://lib.rs/crates/runiq)
- [`uq`](https://lib.rs/crates/uq)
- [`ripuniq`](https://lib.rs/crates/ripuniq)
- [`unic`](https://github.com/donatj/unic)
- [`awk`](https://www.gnu.org/software/gawk/manual/gawk.html)

For the specific commands used please check the [`justfile`](./justfile).

#### Benchmark Table (deduplicate stream)

> Measured on MacBook M3 Pro

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `hist` | 180.6 ± 5.8 | 167.2 | 188.9 | 1.00 |
| `ripuniq` | 215.5 ± 4.7 | 210.4 | 226.2 | 1.19 ± 0.05 |
| `awk` | 1307.1 ± 20.0 | 1289.3 | 1357.0 | 7.24 ± 0.26 |
| `huniq` | 2338.3 ± 52.8 | 2257.4 | 2402.7 | 12.95 ± 0.51 |
| `runiq` | 2413.4 ± 49.0 | 2358.7 | 2535.4 | 13.36 ± 0.51 |
| `uq` | 2942.1 ± 56.0 | 2892.1 | 3068.6 | 16.29 ± 0.61 |
| `unic` | 7915.2 ± 87.0 | 7807.9 | 8034.2 | 43.83 ± 1.49 |
