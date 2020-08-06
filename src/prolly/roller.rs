use fbuzhash::BuzHash;

const DEFAULT_PATTERN: u32 = (1 << 12) - 1;
const DEFAULT_WINDOW_SIZE: u32 = 67;

#[derive(Copy, Clone)]
pub struct Config {
    pub pattern: u32,
    pub window_size: u32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            pattern: DEFAULT_PATTERN,
            window_size: DEFAULT_WINDOW_SIZE,
        }
    }
}
pub struct Roller {
    pattern: u32,
    buzhash: BuzHash,
}
impl Roller {
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }
    pub fn with_config(
        Config {
            pattern,
            window_size,
        }: Config,
    ) -> Self {
        Self {
            pattern,
            buzhash: BuzHash::new(window_size),
        }
    }
    pub fn roll_byte(&mut self, b: u8) -> bool {
        self.buzhash.hash_byte(b) & self.pattern == self.pattern
    }
    pub fn roll_bytes(&mut self, bytes: &[u8]) -> bool {
        for &b in bytes {
            if self.roll_byte(b) {
                return true;
            }
        }
        false
    }
}
