use std::ops::BitXor;

const OS_LEN_BYTES: usize = 256;

#[derive(Clone)]
pub struct Oddsketch(pub [u8; OS_LEN_BYTES]);

impl Oddsketch {
    pub fn insert(&mut self, short_id: u64) {
        let os_index = (short_id % (OS_LEN_BYTES as u64 * 8)) as usize;
        self.0[os_index / 8] ^= 1 << (os_index % 8);
    }

    pub fn hamming_weight(&self) -> u32 {
        self.0.iter().map(|b| b.count_ones()).sum()
    }

    pub fn size(&self) -> u32 {
        let length = 8. * (OS_LEN_BYTES as f64);
        let weight = f64::from(self.hamming_weight());

        let size_approx = f64::ln(1. - 2. * weight / length) / f64::ln(1. - 2. / length);

        size_approx as u32
    }
}

impl BitXor for Oddsketch {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut out = [0; OS_LEN_BYTES];
        for (i, item) in out.iter_mut().enumerate() {
            *item = self.0[i] ^ rhs.0[i];
        }

        Oddsketch(out)
    }
}

impl Default for Oddsketch {
    fn default() -> Self {
        Oddsketch([0; OS_LEN_BYTES])
    }
}
