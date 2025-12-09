use std::borrow::Cow;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::num::NonZeroU64;
use std::ops::Range;
use std::sync::Arc;

use anyhow::Result;
use bstr::io::BufReadExt;
use regex::bytes::Regex;

use crate::bump_bytesmap::BumpBytesMap;
use crate::{Map, Substitute};

struct WorkerRanges<F> {
    reader: BufReader<F>,
    approx_batch_size: i64,
    n_threads: u64,
    n_ranges_yielded: u64,
    eof_pos: u64,
    start: u64,
}

impl<F: Read + Seek> WorkerRanges<F> {
    fn new(file: F, n_threads: NonZeroU64) -> io::Result<Self> {
        let mut reader = BufReader::new(file);
        let eof_pos = reader.seek(SeekFrom::End(0))?;
        reader.rewind()?;
        let approx_batch_size = eof_pos / n_threads;
        // assert: this should almost never be triggered.
        assert!(approx_batch_size <= i64::MAX as u64);
        // such that it's safe to convert to i64 from u64.
        let approx_batch_size = approx_batch_size as i64;
        Ok(Self {
            reader,
            approx_batch_size,
            n_threads: n_threads.get(),
            n_ranges_yielded: 0,
            eof_pos,
            start: 0,
        })
    }

    fn yield_next_range(&mut self) -> io::Result<Option<Range<u64>>> {
        debug_assert_eq!(self.reader.stream_position()?, self.start);
        if self.n_ranges_yielded < self.n_threads && self.start < self.eof_pos {
            let fastforward = (self.approx_batch_size as u64).min(self.eof_pos - self.start);
            let mut next_start = self.start + fastforward;
            self.reader.seek_relative(fastforward as i64)?;
            let n_read = self.reader.skip_until(b'\n')?;
            next_start += n_read as u64;
            if self.n_ranges_yielded + 1 == self.n_threads {
                next_start = self.reader.seek(SeekFrom::End(0))?;
            }
            let range = self.start..next_start;
            self.n_ranges_yielded += 1;
            self.start = next_start;
            debug_assert_eq!(self.reader.stream_position()?, self.start);
            Ok(Some(range))
        } else {
            Ok(None)
        }
    }
}

impl<F: Read + Seek> Iterator for WorkerRanges<F> {
    type Item = io::Result<Range<u64>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.yield_next_range().transpose()
    }
}

fn build_map<R: BufReadExt>(
    reader: &mut R,
    include: Arc<Option<Regex>>,
    exclude: Arc<Option<Regex>>,
    substitutions: Arc<Option<Vec<Substitute>>>,
) -> Result<BumpBytesMap> {
    let mut map = BumpBytesMap::new();
    reader.for_byte_line(|line: &[u8]| {
        // exclude entries on regex match
        if let Some(regex) = exclude.as_ref()
            && regex.is_match(line)
        {
            return Ok(true);
        }

        // include entries on regex match
        if let Some(regex) = include.as_ref()
            && !regex.is_match(line)
        {
            return Ok(true);
        }

        // Perform pattern substitutions per line
        let mut line = Cow::Borrowed(line);
        if let Some(subs) = substitutions.as_ref() {
            for (pat, rep) in subs {
                let new_line = pat.replace_all(&line, rep.as_bytes());
                line = Cow::Owned(new_line.into_owned());
            }
        }

        map.insert_or_inc(line.as_ref());

        Ok(true)
    })?;
    Ok(map)
}

#[derive(Debug)]
pub struct WorkerPanics;

impl fmt::Display for WorkerPanics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("worker thread panics")
    }
}

impl std::error::Error for WorkerPanics {}

pub fn build_maps(
    file: String,
    include: Option<Regex>,
    exclude: Option<Regex>,
    substitutions: Option<Vec<Substitute>>,
    threads: NonZeroU64,
) -> Result<Vec<BumpBytesMap>> {
    let mut wr = WorkerRanges::new(File::open(&file)?, threads)?;
    let mut workers = Vec::new();
    let include = Arc::new(include);
    let exclude = Arc::new(exclude);
    let substitutions = Arc::new(substitutions);
    let file = Arc::new(file);
    while let Some(range) = wr.next() {
        let range = range?;
        let file = Arc::clone(&file);
        let include = Arc::clone(&include);
        let exclude = Arc::clone(&exclude);
        let substitutions = Arc::clone(&substitutions);
        workers.push(std::thread::spawn(move || {
            let mut file = File::open(file.as_ref())?;
            file.seek(SeekFrom::Start(range.start))?;
            let mut reader = BufReader::new(file.take(range.end - range.start));
            build_map(&mut reader, include, exclude, substitutions)
        }))
    }
    let mut maps = Vec::with_capacity(workers.len());
    for hdl in workers {
        maps.push(hdl.join().map_err(|_| WorkerPanics)??);
    }
    Ok(maps)
}

pub fn reduce_maps<'a>(maps: &'a [BumpBytesMap], to_map: &mut Map<'a>) {
    for mp in maps {
        for (k, v) in mp.iter() {
            to_map
                .entry(k)
                .and_modify(|count| *count += v)
                .or_insert(*v);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor};
    use std::num::NonZeroU64;

    use super::WorkerRanges;

    #[test]
    fn test_worker_ranges() -> io::Result<()> {
        let v1 = vec![
            0u8, 0, b'\n', 0, 0, b'\r', b'\n', 0, 0, 0, b'\n', b'\n', b'\n',
        ];
        let v2 = vec![0u8; 9000];
        let v3 = vec![b'\n'];
        let v4 = vec![0u8; 10002];
        let v5 = vec![b'\n'];
        let v6 = vec![0u8; 200];
        let v = [v1, v2, v3, v4, v5, v6].concat();
        assert_eq!(v.len(), 19217);
        // ends: [3u64, 7, 11, 12, 13, 9014, 19017, 19217];

        let f = Cursor::new(&v);
        let mut wr = WorkerRanges::new(f, NonZeroU64::new(1).unwrap())?;
        assert_eq!(wr.next().transpose()?, Some(0..19217));
        assert_eq!(wr.next().transpose()?, None);

        let f = Cursor::new(&v);
        let mut wr = WorkerRanges::new(f, NonZeroU64::new(2).unwrap())?;
        assert_eq!(wr.next().transpose()?, Some(0..19017));
        assert_eq!(wr.next().transpose()?, Some(19017..19217));
        assert_eq!(wr.next().transpose()?, None);

        let f = Cursor::new(&v);
        let mut wr = WorkerRanges::new(f, NonZeroU64::new(3).unwrap())?;
        assert_eq!(wr.next().transpose()?, Some(0..9014));
        assert_eq!(wr.next().transpose()?, Some(9014..19017));
        assert_eq!(wr.next().transpose()?, Some(19017..19217));
        assert_eq!(wr.next().transpose()?, None);

        let f = Cursor::new(&v);
        let mut wr = WorkerRanges::new(f, NonZeroU64::new(3000).unwrap())?;
        assert_eq!(wr.next().transpose()?, Some(0..7));
        assert_eq!(wr.next().transpose()?, Some(7..9014));
        assert_eq!(wr.next().transpose()?, Some(9014..19017));
        assert_eq!(wr.next().transpose()?, Some(19017..19217));
        assert_eq!(wr.next().transpose()?, None);

        let f = Cursor::new(&v);
        let mut wr = WorkerRanges::new(f, NonZeroU64::new(10000).unwrap())?;
        assert_eq!(wr.next().transpose()?, Some(0..3));
        assert_eq!(wr.next().transpose()?, Some(3..7));
        assert_eq!(wr.next().transpose()?, Some(7..11));
        assert_eq!(wr.next().transpose()?, Some(11..13));
        assert_eq!(wr.next().transpose()?, Some(13..9014));
        assert_eq!(wr.next().transpose()?, Some(9014..19017));
        assert_eq!(wr.next().transpose()?, Some(19017..19217));

        Ok(())
    }
}
