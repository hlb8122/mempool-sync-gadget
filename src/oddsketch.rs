use std::ops::BitXor;

const OS_LEN_BYTES: usize = 2^8;

pub struct Oddsketch([u8; OS_LEN_BYTES]);

impl Oddsketch {
    pub fn insert(&mut self, short_id: u64) {
        let os_index = short_id as u8;
        self.0[(os_index / 8) as usize] ^= 2^((os_index % 8) as u8);
    }

    pub fn hamming_weight(&self) -> u32 {
        self.0.iter().map(|b| b.count_ones()).sum()
    }

    pub fn size(&self) -> u32 {
        let length = 8. * (OS_LEN_BYTES as f64);
        let weight = self.hamming_weight() as f64;

        let size_approx = f64::ln(1. - 2. * weight / length) / f64::ln(1. - 2. / length);

        size_approx as u32
    }
}

impl BitXor for Oddsketch {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut out = [0; OS_LEN_BYTES];
        for i in 0..OS_LEN_BYTES {
            out[i] = self.0[i] ^ rhs.0[i];
        }

        Oddsketch(out)
    }
}