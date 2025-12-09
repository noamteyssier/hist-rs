//! Prepare the input for the benchmarks.

use std::io::{self, BufWriter, Read, Seek, Write};
use std::ops::Index;
use std::path::Path;

use hashbrown::HashMap;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use rand_distr::{Distribution, Zipf};
use tempfile::NamedTempFile;

/// The vocabulary size.
const VOCAB_SIZE: usize = 32000;
/// The length in bytes of each token.
const TOKEN_LEN: usize = 16;
/// The random seed for initialization.
const INIT_SEED: u64 = 123456;

/// A vocabulary table.
#[derive(Debug)]
pub struct VocabTable([u8; VOCAB_SIZE * TOKEN_LEN]);

impl VocabTable {
    pub fn new(rng: &mut impl Rng) -> Self {
        let mut data = [0u8; VOCAB_SIZE * TOKEN_LEN];
        rng.fill_bytes(&mut data);
        Self(data)
    }
}

fn half_byte_as_hex(b: u8, hex_buf: &mut u8) {
    match b.checked_sub(10) {
        None => *hex_buf = b'0' + b,
        Some(r) => *hex_buf = b'a' + r,
    }
}

fn byte_as_hex(b: &u8, hex_buf: &mut [u8]) {
    assert_eq!(hex_buf.len(), 2);
    half_byte_as_hex(b >> 4, &mut hex_buf[0]);
    half_byte_as_hex(b & 0x0F, &mut hex_buf[1]);
}

impl VocabTable {
    /// Return the vocabulary size.
    pub const fn len() -> usize {
        VOCAB_SIZE
    }

    /// Get token at `index` as hex string. The hex string will be filled into `hex_buf`. The result
    /// `hex_buf` is guaranteed to contain ASCII characters only. Panics if `index` is not smaller
    /// than `Self::len()`.
    pub fn get_hex(&self, index: usize, hex_buf: &mut [u8; TOKEN_LEN * 2]) {
        for (i, b) in self[index].iter().enumerate() {
            byte_as_hex(b, &mut hex_buf[2 * i..2 * (i + 1)]);
        }
    }
}

impl Index<usize> for VocabTable {
    type Output = [u8];

    /// Get the token bytes at `index`. Panics if `index` is not smaller than `Self::len()`.
    fn index(&self, index: usize) -> &Self::Output {
        let start = TOKEN_LEN * index;
        &self.0[start..start + TOKEN_LEN]
    }
}

/// Sample token id from `distribution` until the id falls in valid vocab table index range.
fn sample_token_id(distribution: &impl Distribution<f64>, rng: &mut impl Rng) -> usize {
    loop {
        let index = rng.sample(distribution).floor() as usize;
        if index < VocabTable::len() {
            break index;
        }
    }
}

/// The input data as a temporary file.
#[derive(Debug)]
pub struct InputData {
    tempfile: NamedTempFile,
    // Used in integration tests.
    #[allow(unused)]
    pub groundtruth: HashMap<Vec<u8>, usize>,
}

impl InputData {
    /// Create a temp input data containing `lines` lines of random tokens.
    pub fn new(lines: u64) -> io::Result<Self> {
        let mut writer = BufWriter::new(NamedTempFile::new_in(env!("CARGO_MANIFEST_DIR"))?);
        let zf = Zipf::new(VOCAB_SIZE as f64, 1.0).expect("failed to init Zipfian distribution");
        let mut hex_buf = [0u8; TOKEN_LEN * 2];
        let mut rng = ChaCha12Rng::seed_from_u64(INIT_SEED);
        let vocab = VocabTable::new(&mut rng);
        let mut groundtruth = HashMap::new();
        for _ in 0..lines {
            let token_id = sample_token_id(&zf, &mut rng);
            vocab.get_hex(token_id, &mut hex_buf);
            writer.write_all(&hex_buf)?;
            writer.write_all(b"\n")?;
            groundtruth
                .entry(hex_buf.to_vec())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
        let mut file = writer.into_inner()?;
        file.rewind()?;
        Ok(Self {
            tempfile: file,
            groundtruth,
        })
    }

    /// Get the path to the temporary input data.
    pub fn path(&self) -> &Path {
        &self.tempfile.path()
    }

    /// Close and clean up the temporary input data.
    pub fn close(self) -> io::Result<()> {
        self.tempfile.close()
    }
}

impl Read for InputData {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.tempfile.read(buf)
    }
}

impl Seek for InputData {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.tempfile.seek(pos)
    }
}
