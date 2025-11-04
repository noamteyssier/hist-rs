mod cli;
use cli::Args;

use std::{
    borrow::Cow,
    cmp::Ordering,
    io::{self, Write},
};

use anyhow::Result;
use bstr::io::BufReadExt;
use bumpalo::Bump;
use clap::Parser;
use hashbrown::{HashMap, HashSet, hash_map::RawEntryMut};
use regex::bytes::Regex;

type Set<'a> = HashSet<&'a [u8]>;
type Map<'a> = HashMap<&'a [u8], usize>;
type FlatCounts<'a> = Vec<(&'a [u8], usize)>;
type Substitute<'a> = (Regex, &'a [u8]);

fn build_map<'a, R: BufReadExt>(
    reader: &mut R,
    map: &mut Map<'a>,
    arena: &'a Bump,
    include: Option<Regex>,
    exclude: Option<Regex>,
    substitutions: Option<&[Substitute<'_>]>,
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

        // Perform pattern substitutions per line
        let mut line = Cow::Borrowed(line);
        if let Some(subs) = substitutions {
            for (pat, rep) in subs {
                let new_line = pat.replace_all(&line, *rep);
                line = Cow::Owned(new_line.into_owned());
            }
        }

        // manual entry handling
        match map.raw_entry_mut().from_key(line.as_ref()) {
            // exists - increment count
            RawEntryMut::Occupied(mut entry) => {
                *entry.get_mut() += 1;
            }
            // new entry - allocate into arena and insert slice
            RawEntryMut::Vacant(entry) => {
                let owned = arena.alloc_slice_copy(line.as_ref());
                entry.insert(owned, 1);
            }
        }

        Ok(true)
    })?;
    Ok(())
}

fn stream_unique<R: BufReadExt, W: Write>(
    reader: &mut R,
    writer: &mut W,
    arena: &'_ Bump,
    include: Option<Regex>,
    exclude: Option<Regex>,
    substitutions: Option<&[Substitute<'_>]>,
) -> Result<()> {
    let mut csv_writer = csv::Writer::from_writer(writer);
    let mut set = Set::default();
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

        // Perform pattern substitutions per line
        let mut line = Cow::Borrowed(line);
        if let Some(subs) = substitutions {
            for (pat, rep) in subs {
                let new_line = pat.replace_all(&line, *rep);
                line = Cow::Owned(new_line.into_owned());
            }
        }

        if !set.contains(line.as_ref()) {
            let owned = arena.alloc_slice_copy(line.as_ref());

            // Safety: This is safe because we've already hashed the line into the set and it is not present
            unsafe {
                set.insert_unique_unchecked(owned);
            }

            let line_str = std::str::from_utf8(line.as_ref()).map_err(io::Error::other)?;
            csv_writer.serialize(line_str)?;
        }

        Ok(true)
    })?;
    csv_writer.flush()?;
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
            let record: (usize, &str) = (value, std::str::from_utf8(key)?);
            writer.serialize(record)?;
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
                writer.serialize(other_sum_record)?;
                other_sum_written = true;
            }

            // write the current record
            let record: (usize, &str) = (value, std::str::from_utf8(key)?);
            writer.serialize(record)?;
            Ok(())
        })?;

    // write the other sum record if it hasn't been written yet
    if !other_sum_written {
        writer.serialize(other_sum_record)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut in_handle = args.match_input()?;
    let mut out_handle = args.match_output()?;

    let arena = Bump::new();

    if args.unique {
        stream_unique(
            &mut in_handle,
            &mut out_handle,
            &arena,
            args.include_regex()?,
            args.exclude_regex()?,
            args.substitutes()?.as_deref(),
        )?;
    } else {
        let mut map = Map::default();

        build_map(
            &mut in_handle,
            &mut map,
            &arena,
            args.include_regex()?,
            args.exclude_regex()?,
            args.substitutes()?.as_deref(),
        )?;

        let sorted_collection =
            sort_collection(map, args.descending, args.skip_sorting, args.sort_by_name);

        if args.last_k() > 0 {
            write_topk_flatcounts(&mut out_handle, sorted_collection, args.last_k())?;
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
