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
- [`awk`](https://www.gnu.org/software/gawk/manual/gawk.html) ( `awk '{ x[$0]++ } END { for(y in x) { print y, x[y] }}' <file | sort -k2,2nr` )
- Naive Implementation (coreutils `sort <file | uniq -c | sort -n`)
- Naive no cache (LC_ALL=C) (`export LC_ALL=C; sort -n <file | uniq -c | sort -n`)
- Naive no cache size hints (LC_ALL=C) (`export LC_ALL=C; sort -S 1G -n <file | uniq -c | sort -S 1G -n`)

### Benchmark Table

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `hist` | 238.4 ± 3.9 | 233.2 | 246.8 | 1.00 |
| `cuniq` | 537.4 ± 5.5 | 531.2 | 547.0 | 2.25 ± 0.04 |
| `naive-no-cache-size-hints` | 1184.2 ± 27.6 | 1146.1 | 1223.1 | 4.97 ± 0.14 |
| `naive-no-cache` | 1198.2 ± 22.7 | 1146.6 | 1220.6 | 5.03 ± 0.13 |
| `awk` | 1245.0 ± 1.8 | 1242.3 | 1248.7 | 5.22 ± 0.09 |
| `huniq` | 2896.9 ± 36.9 | 2854.5 | 2988.6 | 12.15 ± 0.25 |
| `sortuniq` | 3290.3 ± 97.9 | 3173.1 | 3517.5 | 13.80 ± 0.47 |
| `naive` | 5594.4 ± 53.5 | 5524.5 | 5723.8 | 23.47 ± 0.45 |
