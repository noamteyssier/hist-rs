use std::{
    fs::File,
    io::{BufReader, BufWriter, Write, stdin, stdout},
};

use anyhow::Result;
use bstr::io::BufReadExt;
use clap::Parser;
use hashbrown::HashMap;

fn build_map<R: BufReadExt>(reader: &mut R, map: &mut HashMap<Vec<u8>, usize>) -> Result<()> {
    reader.for_byte_line(|line: &[u8]| {
        *map.entry_ref(line).or_default() += 1;
        Ok(true)
    })?;
    Ok(())
}

fn sort_collection(map: HashMap<Vec<u8>, usize>) -> Vec<(Vec<u8>, usize)> {
    let mut collection = map.into_iter().collect::<Vec<_>>();
    collection.sort_unstable_by(|a, b| a.1.cmp(&b.1));
    collection
}

fn write_collection<W: Write>(wtr: &mut W, collection: Vec<(Vec<u8>, usize)>) -> Result<()> {
    let mut writer = csv::WriterBuilder::new().delimiter(b'\t').from_writer(wtr);
    collection
        .into_iter()
        .try_for_each(|(key, value)| -> Result<()> {
            let record: (usize, &str) = (value, std::str::from_utf8(&key)?);
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
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut in_handle = args.match_input()?;
    let mut out_handle = args.match_output()?;
    let mut map: HashMap<Vec<u8>, usize> = HashMap::new();

    build_map(&mut in_handle, &mut map)?;
    let sorted_collection = sort_collection(map);
    write_collection(&mut out_handle, sorted_collection)?;

    Ok(())
}
