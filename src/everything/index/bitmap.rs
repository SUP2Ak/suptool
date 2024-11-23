use std::ops::{BitAnd, BitOr, BitOrAssign};
use std::io::{self, Read, Write};

#[derive(Clone, Copy)]
pub(crate) struct CharBitmap {
    words: [u64; 4]  // 256 bits pour tous les caractères ASCII
}

impl CharBitmap {
    pub fn new() -> Self {
        Self { words: [0; 4] }
    }

    #[inline]
    pub fn set(&mut self, byte: u8) {
        let word_idx = (byte >> 6) as usize;
        let bit_pos = byte & 0x3F;
        self.words[word_idx] |= 1u64 << bit_pos;
    }

    #[inline]
    pub fn test(&self, byte: u8) -> bool {
        let word_idx = (byte >> 6) as usize;
        let bit_pos = byte & 0x3F;
        (self.words[word_idx] & (1u64 << bit_pos)) != 0
    }

    #[inline]
    pub fn set_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            self.set(b);
        }
    }

    #[inline]
    pub fn contains_pattern(&self, pattern: &[u8]) -> bool {
        pattern.iter().all(|&b| self.test(b))
    }

    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                self.words.as_ptr() as *const u8,
                std::mem::size_of::<[u64; 4]>()
            )
        };
        writer.write_all(bytes)
    }

    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut words = [0u64; 4];
        for word in &mut words {
            let mut bytes = [0u8; 8];
            reader.read_exact(&mut bytes)?;
            *word = u64::from_le_bytes(bytes);
        }
        Ok(Self { words })
    }

    pub fn get(&self, c: u8) -> bool {
        if c >= 128 {
            return false;
        }
        let idx = (c >> 5) as usize;
        let bit = c & 0x1F;
        if idx >= self.words.len() {
            return false;
        }
        (self.words[idx] & (1 << bit)) != 0
    }

    pub fn contains_all(&self, other: &CharBitmap) -> bool {
        for (self_word, other_word) in self.words.iter().zip(other.words.iter()) {
            // Si des bits sont présents dans other mais pas dans self, retourner false
            if (*other_word & *self_word) != *other_word {
                return false;
            }
        }
        true
    }
}

impl BitAnd for &CharBitmap {
    type Output = CharBitmap;

    fn bitand(self, rhs: &CharBitmap) -> CharBitmap {
        let mut result = CharBitmap::new();
        for i in 0..4 {
            result.words[i] = self.words[i] & rhs.words[i];
        }
        result
    }
}

impl BitOr for &CharBitmap {
    type Output = CharBitmap;

    fn bitor(self, rhs: &CharBitmap) -> CharBitmap {
        let mut result = CharBitmap::new();
        for i in 0..4 {
            result.words[i] = self.words[i] | rhs.words[i];
        }
        result
    }
}

impl BitOrAssign<&CharBitmap> for CharBitmap {
    fn bitor_assign(&mut self, rhs: &CharBitmap) {
        for i in 0..4 {
            self.words[i] |= rhs.words[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_operations() {
        let mut bm1 = CharBitmap::new();
        let mut bm2 = CharBitmap::new();

        bm1.set_str("hello");
        bm2.set_str("world");

        // Test AND
        let and = &bm1 & &bm2;
        assert!(and.test(b'l'));
        assert!(and.test(b'o'));
        assert!(!and.test(b'h'));
        assert!(!and.test(b'w'));

        // Test OR
        let or = &bm1 | &bm2;
        assert!(or.test(b'h'));
        assert!(or.test(b'w'));
        assert!(or.test(b'l'));
        assert!(or.test(b'o'));
        assert!(or.test(b'd'));
    }

    #[test]
    fn test_pattern_matching() {
        let mut bm = CharBitmap::new();
        bm.set_str("hello world");
        
        assert!(bm.contains_pattern(b"hell"));
        assert!(bm.contains_pattern(b"world"));
        assert!(!bm.contains_pattern(b"xyz"));
    }
}