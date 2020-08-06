// #[cfg(test)]
const CHUNK_PATTERN: u32 = 1 << 8 - 1;

/// An eventual abstraction over the real rolled hashing impl.
///
/// Right now though, it's using a fake rolled hashing. Assume very poor performance.
/// We'll add a buzhash eventually.
///
/// This is a really, really bad impl, on purpose. Need a proper roll.
pub struct Roller {
    window_size: usize,
    bytes: Vec<u8>,
}
impl Roller {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            bytes: Vec::new(),
        }
    }
    pub fn roll_byte(&mut self, b: u8) -> bool {
        self.bytes.push(b);
        if self.bytes.len() > self.window_size {
            self.bytes.remove(0);
        }
        // super silly, but just making it compile before buzhash.
        let hash = <[u8; 32]>::from(blake3::hash(&self.bytes));
        let hash = u32::from_ne_bytes([hash[0], hash[1], hash[2], hash[3]]);
        hash & CHUNK_PATTERN == CHUNK_PATTERN
    }
    pub fn roll_bytes(&mut self, bytes: &[u8]) -> bool {
        for &b in bytes {
            if !self.roll_byte(b) {
                return false;
            }
        }
        true
    }
}
