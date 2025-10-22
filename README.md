# hist

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.md)
[![Crates.io](https://img.shields.io/crates/d/hist-rs?color=orange&label=crates.io)](https://crates.io/crates/hist-rs)

A high-throughput CLI to count unique lines.

This is a standalone tool with equivalent functionality to `cat <file> | sort | uniq -c | sort -n`.

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
```

## Benchmarks

I use [`nucgen`](https://crates.io/crates/nucgen) to generate a random 100M line [FASTQ file](https://en.wikipedia.org/wiki/FASTQ_format) and pipe it into different tools to compare their throughput with [`hyperfine`](https://lib.rs/crates/hyperfine).

I am measuring the performance of equivalent `cat <file> | sort | uniq -c | sort -n` functionality.

Tools compared:
- [`hist`](https://lib.rs/crates/hist-rs)
- [`cuniq`](https://lib.rs/crates/cuniq)
- [`huniq`](https://lib.rs/crates/huniq)
- [`sortuniq`](https://lib.rs/crates/sortuniq)
- Naive Implementation (coreutils `cat <file> | sort | uniq -c | sort -n`)

### Benchmark Table

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `hist` | 200.3 ± 3.3 | 195.6 | 208.7 | 1.00 |
| `cuniq` | 434.3 ± 6.6 | 424.7 | 442.9 | 2.17 ± 0.05 |
| `huniq` | 2375.5 ± 43.8 | 2328.1 | 2450.3 | 11.86 ± 0.30 |
| `sortuniq` | 2593.2 ± 28.4 | 2535.7 | 2640.9 | 12.95 ± 0.26 |
| `naive` | 5409.9 ± 23.3 | 5378.0 | 5453.3 | 27.01 ± 0.47 |
