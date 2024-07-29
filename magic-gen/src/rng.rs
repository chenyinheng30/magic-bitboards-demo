// Simple Pcg64Mcg implementation
pub struct Rng(u128);

impl Default for Rng {
    fn default() -> Self {
        Self(0xE926E6210D9E3486 | 1)
    }
}

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(0x2360ED051FC65DA44385DF649FCCF645);
        let rot = (self.0 >> 122) as u32;
        let xsl = (self.0 >> 64) as u64 ^ self.0 as u64;
        xsl.rotate_right(rot)
    }
    pub fn next_u128(&mut self) -> u128 {
        let x = self.next_u64() as u128;
        let y = self.next_u64() as u128;
        (x << 64) | y
    }
}
