use std::{
    fs::File,
    io::{BufReader, BufWriter, Write, stdin, stdout},
};

use anyhow::Result;

use bstr::io::BufReadExt;
use clap::Parser;
use regex::bytes::Regex;

#[derive(Parser)]
pub struct Args {
    input: Option<String>,

    output: Option<String>,

    /// Skip counting and just write unique lines in the same order as input
    #[clap(short = 'u', long)]
    pub unique: bool,

    /// Only include incoming entries that match a regex pattern
    #[clap(short = 'i', long)]
    pub include: Option<String>,

    /// Exclude incoming entries that match a regex pattern
    #[clap(short = 'e', long)]
    pub exclude: Option<String>,

    /// Filter out entries with abundance less than MIN
    #[clap(short = 'm', long)]
    pub min: Option<usize>,

    /// Filter out entries with abundance greater than MAX
    #[clap(short = 'M', long)]
    pub max: Option<usize>,

    /// Sort descending by abundance
    #[clap(short = 'd', long)]
    pub descending: bool,

    /// Skip sorting
    #[clap(short = 's', long, conflicts_with = "descending")]
    pub skip_sorting: bool,

    /// Sort by entry name
    #[clap(short = 'n', long)]
    pub sort_by_name: bool,

    /// Shows the last-k entries and a count of the other entries
    #[clap(short = 'k', long, conflicts_with_all = ["min", "max", "skip_sorting"])]
    last_k: Option<usize>,
}
impl Args {
    pub fn match_input(&self) -> Result<Box<dyn BufReadExt>> {
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

    pub fn match_output(&self) -> Result<Box<dyn Write>> {
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

    pub fn include_regex(&self) -> Result<Option<Regex>> {
        if let Some(pattern) = &self.include {
            Ok(Some(Regex::new(pattern)?))
        } else {
            Ok(None)
        }
    }

    pub fn exclude_regex(&self) -> Result<Option<Regex>> {
        if let Some(pattern) = &self.exclude {
            Ok(Some(Regex::new(pattern)?))
        } else {
            Ok(None)
        }
    }

    pub fn last_k(&self) -> usize {
        self.last_k.unwrap_or(0)
    }
}
