use std::{
    cmp::Ordering,
    fs::File,
    io::{BufReader, BufWriter, Write, stdin, stdout},
    usize,
};

use anyhow::Result;
use bstr::io::BufReadExt;
use bumpalo::Bump;
use clap::Parser;
use hashbrown::{HashMap, hash_map::RawEntryMut};
use regex::bytes::Regex;

type Map<'a> = HashMap<&'a [u8], usize>;
type FlatCounts<'a> = Vec<(&'a [u8], usize)>;

fn build_map<'a, R: BufReadExt>(
    reader: &mut R,
    map: &mut Map<'a>,
    arena: &'a Bump,
    include: Option<Regex>,
    exclude: Option<Regex>,
) -> Result<()> {
    reader.for_byte_line(|line: &[u8]| {
        // exclude entries on regex match
        if let Some(ref regex) = exclude
            && regex.is_match(line)
        {
            return Ok(true);
        }

        // include entries on regex match
        if let Some(ref regex) = include
            && !regex.is_match(line)
        {
            return Ok(true);
        }

        // manual entry handling
        match map.raw_entry_mut().from_key(line) {
            // exists - increment count
            RawEntryMut::Occupied(mut entry) => {
                *entry.get_mut() += 1;
            }
            // new entry - allocate into arena and insert slice
            RawEntryMut::Vacant(entry) => {
                let owned = arena.alloc_slice_copy(line);
                entry.insert(owned, 1);
            }
        }

        Ok(true)
    })?;
    Ok(())
}

fn sort_func<T: Ord>(a: &T, b: &T) -> Ordering {
    a.cmp(b)
}

fn sort_collection(
    map: Map,
    descending: bool,
    skip_sorting: bool,
    sort_by_name: bool,
) -> FlatCounts {
    let mut collection = map.into_iter().collect::<Vec<_>>();
    if skip_sorting {
        collection
    } else {
        match (sort_by_name, descending) {
            (true, false) => collection.sort_unstable_by(|a, b| sort_func(&a.0, &b.0)),
            (true, true) => collection.sort_unstable_by(|a, b| sort_func(&b.0, &a.0)),
            (false, false) => collection.sort_unstable_by(|a, b| sort_func(&a.1, &b.1)),
            (false, true) => collection.sort_unstable_by(|a, b| sort_func(&b.1, &a.1)),
        }
        collection
    }
}

fn write_flatcounts<W: Write>(
    wtr: &mut W,
    collection: FlatCounts,
    max: usize,
    min: usize,
) -> Result<()> {
    let mut writer = csv::WriterBuilder::new().delimiter(b'\t').from_writer(wtr);
    collection
        .into_iter()
        .filter(|(_, value)| *value <= max && *value >= min)
        .try_for_each(|(key, value)| -> Result<()> {
            let record: (usize, &str) = (value, std::str::from_utf8(&key)?);
            writer.serialize(&record)?;
            Ok(())
        })?;
    writer.flush()?;
    Ok(())
}

fn write_topk_flatcounts<W: Write>(wtr: &mut W, collection: FlatCounts, k: usize) -> Result<()> {
    let mut writer = csv::WriterBuilder::new().delimiter(b'\t').from_writer(wtr);
    // set the left-bound for the top-k elements
    let lbound = collection.len().saturating_sub(k);

    // sums the counts of the bottom-(len-k) elements
    let other_sum: usize = collection.iter().take(lbound).map(|(_, value)| value).sum();
    let other_sum_record = (other_sum, &format!("other lines ({} elements)", lbound));
    let mut other_sum_written = false;

    // Write all records except the bottom-(len-k) elements
    collection
        .into_iter()
        .skip(lbound)
        .try_for_each(|(key, value)| -> Result<()> {
            // place the other sum record if it hasn't been written yet
            if !other_sum_written && value > other_sum {
                writer.serialize(&other_sum_record)?;
                other_sum_written = true;
            }

            // write the current record
            let record: (usize, &str) = (value, std::str::from_utf8(&key)?);
            writer.serialize(&record)?;
            Ok(())
        })?;

    // write the other sum record if it hasn't been written yet
    if !other_sum_written {
        writer.serialize(&other_sum_record)?;
    }

    writer.flush()?;
    Ok(())
}

fn write_entries<W: Write>(wtr: &mut W, map: &Map) -> Result<()> {
    let mut writer = csv::WriterBuilder::new().delimiter(b'\t').from_writer(wtr);
    map.iter().try_for_each(|(key, _)| -> Result<()> {
        let record: &str = std::str::from_utf8(key)?;
        writer.serialize(&record)?;
        Ok(())
    })?;
    writer.flush()?;
    Ok(())
}

#[derive(Parser)]
struct Args {
    input: Option<String>,

    output: Option<String>,

    /// Skip counting and just write unique lines
    #[clap(short, long)]
    unique: bool,

    /// Only include incoming entries that match a regex pattern
    #[clap(short = 'i', long)]
    include: Option<String>,

    /// Exclude incoming entries that match a regex pattern
    #[clap(short = 'e', long)]
    exclude: Option<String>,

    /// Filter out entries with abundance less than MIN
    #[clap(short = 'm', long)]
    min: Option<usize>,

    /// Filter out entries with abundance greater than MAX
    #[clap(short = 'M', long)]
    max: Option<usize>,

    /// Sort descending by abundance
    #[clap(short = 'd', long)]
    descending: bool,

    /// Skip sorting
    #[clap(short = 's', long, conflicts_with = "descending")]
    skip_sorting: bool,

    /// Sort by entry name
    #[clap(short = 'n', long)]
    sort_by_name: bool,

    /// Shows the last-k entries and a count of the other entries
    #[clap(short = 'k', long, conflicts_with_all = ["min", "max", "skip_sorting"])]
    k: Option<usize>,
}
impl Args {
    fn match_input(&self) -> Result<Box<dyn BufReadExt>> {
        match &self.input {
            Some(path) => {
                let handle = File::open(path).map(BufReader::new)?;
                Ok(Box::new(handle))
            }
            None => {
                let handle = BufReader::new(stdin());
                Ok(Box::new(handle))
            }
        }
    }

    fn match_output(&self) -> Result<Box<dyn Write>> {
        match &self.output {
            Some(path) => {
                let handle = File::create(path).map(BufWriter::new)?;
                Ok(Box::new(handle))
            }
            None => {
                let handle = BufWriter::new(stdout());
                Ok(Box::new(handle))
            }
        }
    }

    fn include_regex(&self) -> Result<Option<Regex>> {
        if let Some(pattern) = &self.include {
            Ok(Some(Regex::new(pattern)?))
        } else {
            Ok(None)
        }
    }

    fn exclude_regex(&self) -> Result<Option<Regex>> {
        if let Some(pattern) = &self.exclude {
            Ok(Some(Regex::new(pattern)?))
        } else {
            Ok(None)
        }
    }

    fn k(&self) -> usize {
        self.k.unwrap_or(0)
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut in_handle = args.match_input()?;
    let mut out_handle = args.match_output()?;

    let arena = Bump::new();
    let mut map = Map::default();

    build_map(
        &mut in_handle,
        &mut map,
        &arena,
        args.include_regex()?,
        args.exclude_regex()?,
    )?;

    if args.unique {
        write_entries(&mut out_handle, &map)?;
    } else {
        let sorted_collection =
            sort_collection(map, args.descending, args.skip_sorting, args.sort_by_name);

        if args.k() > 0 {
            write_topk_flatcounts(&mut out_handle, sorted_collection, args.k())?;
        } else {
            write_flatcounts(
                &mut out_handle,
                sorted_collection,
                args.max.unwrap_or(usize::MAX),
                args.min.unwrap_or(0),
            )?;
        }
    }

    Ok(())
}
