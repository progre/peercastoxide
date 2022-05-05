use log::*;
use once_cell::sync::Lazy;
use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

static COLORS: Lazy<Vec<u8>> = Lazy::new(|| {
    (0..0xff)
        .filter(|&x| x != 0 && !(0x10..=0x15).contains(&x) && !(0xe8..=0xeb).contains(&x))
        .collect()
});

pub struct ConsoleColor {
    color_header: String,
}

impl ConsoleColor {
    pub fn random_color(seed: &str) -> Self {
        let mut hash = DefaultHasher::new();
        seed.hash(&mut hash);
        let seed_u64 = hash.finish();
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed_u64);
        trace!("{} * {} / {}", COLORS.len(), rng.next_u32(), u32::MAX);
        let idx = (COLORS.len() as f64 * (rng.next_u32() as f64) / u32::MAX as f64) as usize;
        Self::new(COLORS[idx])
    }

    pub fn new(color: u8)  -> Self{
        Self {
            color_header: format!("\x1b[38;5;{}m", color),
        }
    }

    pub fn header(&self) -> &str {
        &self.color_header
    }

    pub fn footer(&self) -> &str {
        "\x1b[m"
    }
}
