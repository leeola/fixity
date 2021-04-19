use fbuzhash::BuzHash;

const DEFAULT_PATTERN: u32 = (1 << 12) - 1;
const DEFAULT_WINDOW_SIZE: u32 = 67;

#[derive(Copy, Clone)]
pub struct Config {
    pub pattern: u32,
    pub window_size: u32,
}
impl Config {
    /// The pattern to look for in the [`Roller`].
    ///
    /// Typically a series of all `1` bits, with the width indicating the probability.
    ///
    /// See also: [`Roller::roll_byte`].
    pub fn with_pattern(pattern: u32) -> Self {
        Self {
            pattern,
            window_size: DEFAULT_WINDOW_SIZE,
        }
    }
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
    /// Roll the given byte and return whether or not it matches the underlying pattern.
    ///
    /// # High Level Explanation
    ///
    /// Two primary principles are at play with the `Roller`.
    ///
    /// 1. Hashing of input data, a single byte in this case.
    /// 2. Checking if the new rolling hash result matches a pattern.
    ///
    /// Hashing is basic, we expect a evenly distributed hash based on the input data.
    /// Super standard.
    ///
    /// The pattern matching however is written in such a way that the "pattern" is used to
    /// effectively truncate the resulting hash. The truncated value is then equality compared to
    /// the pattern, and if matching, the input byte matched the pattern.
    ///
    /// This truncate and match allows the pattern to be of a variable size and complexity. This is
    /// useful to easily change the likihood of a pattern match. The smaller the pattern the
    /// more likely a pattern match is found.
    ///
    /// # Pattern Matching
    ///
    /// Pattern matching is a basic Bitwise `AND` to truncate the hash by the pattern, which are
    /// expected to be a high bit set of `1`s. Eg a pattern of `0b111` would truncate `0b10111`
    /// to `0b101`. This behavior means that a pattern match of `0b111` has a 1 in 8 chance of
    /// occuring, with a probability of `1/2^8`. The probability is thereby configurable based
    /// on the chosen bit width, aka 8 in that example. A larger bit width would be less likely
    /// to occur, resulting in wider chunk sizes.
    pub fn roll_byte(&mut self, b: u8) -> bool {
        self.buzhash.hash_byte(b) & self.pattern == self.pattern
    }
    /// Roll the given bytes and return whether or not it matches the underlying pattern.
    ///
    /// For conceptual documentation, see the single-byte version of this method,
    /// [`Roller::roll_byte`].
    pub fn roll_bytes(&mut self, bytes: &[u8]) -> bool {
        for &b in bytes {
            if self.roll_byte(b) {
                return true;
            }
        }
        false
    }
}
impl Default for Roller {
    fn default() -> Self {
        Self::new()
    }
}
