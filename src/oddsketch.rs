use std::ops::{BitXor, BitXorAssign, Deref};

const OS_LEN_BYTES: usize = 256;

#[derive(Clone)]
pub struct Oddsketch([u8; OS_LEN_BYTES]);

impl Oddsketch {
    pub fn insert(&mut self, short_id: u64) {
        let os_index = (short_id % (OS_LEN_BYTES as u64 * 8)) as usize;
        self.0[os_index / 8] ^= 1 << (os_index % 8);
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|x| *x == 0)
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

impl BitXorAssign for Oddsketch {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (i, item) in self.0.iter_mut().enumerate() {
            *item ^= rhs.0[i];
        }
    }
}

impl Default for Oddsketch {
    fn default() -> Self {
        Oddsketch([0; OS_LEN_BYTES])
    }
}

impl Deref for Oddsketch {
    type Target = [u8; OS_LEN_BYTES];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
