use anyhow::Result;
use hashbrown::HashMap;
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, stdin},
};

fn main() -> Result<()> {
    let input = stdin();
    let mut reader = BufReader::new(input);
    let mut bufstr = String::default();
    let mut map: HashMap<String, usize> = HashMap::new();

    while reader.read_line(&mut bufstr)? > 0 {
        let key = bufstr.trim();
        if key.is_empty() {
            continue;
        }
        *map.entry_ref(key).or_default() += 1;
        bufstr.clear();
    }

    let btree = BTreeMap::from_iter(map);
    btree.into_iter().for_each(|(key, value)| {
        println!("{}\t{}", value, key);
    });

    Ok(())
}
