use std::{
    cmp::Ordering,
    fs::File,
    io::{BufReader, BufWriter, Write, stdin, stdout},
    usize,
};

use anyhow::Result;
use bstr::io::BufReadExt;
use clap::Parser;
use hashbrown::HashMap;
use regex::bytes::Regex;

type Map = HashMap<Vec<u8>, usize>;
type FlatCounts = Vec<(Vec<u8>, usize)>;

fn build_map<R: BufReadExt>(
    reader: &mut R,
    map: &mut Map,
    include: Option<Regex>,
    exclude: Option<Regex>,
) -> Result<()> {
    reader.for_byte_line(|line: &[u8]| {
        if let Some(ref regex) = exclude
            && regex.is_match(line)
        {
            return Ok(true);
        }
        if let Some(ref regex) = include
            && !regex.is_match(line)
        {
            return Ok(true);
        }
        *map.entry_ref(line).or_default() += 1;
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
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut in_handle = args.match_input()?;
    let mut out_handle = args.match_output()?;
    let mut map = HashMap::new();

    build_map(
        &mut in_handle,
        &mut map,
        args.include_regex()?,
        args.exclude_regex()?,
    )?;

    if args.unique {
        write_entries(&mut out_handle, &map)?;
    } else {
        let sorted_collection =
            sort_collection(map, args.descending, args.skip_sorting, args.sort_by_name);
        write_flatcounts(
            &mut out_handle,
            sorted_collection,
            args.max.unwrap_or(usize::MAX),
            args.min.unwrap_or(0),
        )?;
    }

    Ok(())
}
