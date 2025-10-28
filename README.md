# hist

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.md)
[![Crates.io](https://img.shields.io/crates/d/hist-rs?color=orange&label=crates.io)](https://crates.io/crates/hist-rs)

A high-throughput CLI to count unique lines.

This is a standalone tool with equivalent functionality to `sort | uniq -c | sort -n`.

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

# skip counting and just write unique lines
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

I use [`nucgen`](https://crates.io/crates/nucgen) to generate a random 1M line [FASTQ file](https://en.wikipedia.org/wiki/FASTQ_format) and pipe it into different tools to compare their throughput with [`hyperfine`](https://lib.rs/crates/hyperfine).

I am measuring the performance of equivalent `sort <file | uniq -c | sort -n` functionality.

Tools compared:
- [`hist`](https://lib.rs/crates/hist-rs)
- [`cuniq`](https://lib.rs/crates/cuniq)
- [`huniq`](https://lib.rs/crates/huniq)
- [`sortuniq`](https://lib.rs/crates/sortuniq)
- [`awk`](https://www.gnu.org/software/gawk/manual/gawk.html)
- Naive Implementation (coreutils `sort <file | uniq -c | sort -n`)
- Naive no cache (LC_ALL=C)
- Naive no cache size hints (LC_ALL=C; size hints for `sort`)

For the specific commands used please check the [`justfile`](./justfile).

### Benchmark Table

> Measured on MacBook M3 Pro

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `hist` | 228.1 ± 7.7 | 223.9 | 252.6 | 1.00 |
| `cuniq` | 532.2 ± 11.0 | 514.0 | 554.9 | 2.33 ± 0.09 |
| `naive-no-locale-size-hints` | 1164.7 ± 32.1 | 1119.3 | 1227.2 | 5.11 ± 0.22 |
| `naive-no-locale` | 1180.7 ± 32.7 | 1128.6 | 1228.7 | 5.18 ± 0.23 |
| `awk` | 1248.1 ± 18.8 | 1239.1 | 1300.6 | 5.47 ± 0.20 |
| `huniq` | 2812.5 ± 83.7 | 2721.0 | 2957.6 | 12.33 ± 0.56 |
| `sortuniq` | 3108.2 ± 34.8 | 3056.8 | 3168.2 | 13.63 ± 0.48 |
| `naive` | 5591.6 ± 89.7 | 5478.3 | 5717.0 | 24.52 ± 0.92 |
| `naive-size-hints` | 5628.2 ± 155.1 | 5444.7 | 5863.3 | 24.68 ± 1.08 |
