use anyhow::Result;
use bstr::io::BufReadExt;
use hashbrown::HashMap;
use std::io::{BufReader, stdin};

fn main() -> Result<()> {
    let input = stdin();
    let mut reader = BufReader::new(input);
    let mut map: HashMap<Vec<u8>, usize> = HashMap::new();

    reader.for_byte_line(|line: &[u8]| {
        *map.entry_ref(line).or_default() += 1;
        Ok(true)
    })?;

    let mut collection = map.into_iter().collect::<Vec<_>>();
    collection.sort_unstable_by(|a, b| a.1.cmp(&b.1));

    let out_handle = std::io::stdout();
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(out_handle);
    collection
        .into_iter()
        .try_for_each(|(key, value)| -> Result<()> {
            let record: (usize, &str) = (value, std::str::from_utf8(&key)?);
            writer.serialize(&record)?;
            Ok(())
        })?;

    Ok(())
}
