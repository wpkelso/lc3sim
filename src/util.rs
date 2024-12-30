use crate::defs::LC3Word;

/// Format bits from a single byte into a nice to read format.
pub fn format_bits(byte: u8) -> String {
    (0..8)
        .rev()
        .map(|mask_bit| 1 << mask_bit)
        .map(|mask| if (byte & mask) != 0 { '1' } else { '0' })
        .collect()
}

/// Format bits from `idx` byte of a LC3 word into a nice to read format.
pub fn format_word_bits(word: LC3Word, idx: usize) -> String {
    format_bits(word.to_be_bytes()[idx])
}

/// Format all bits from `idx` byte of a LC3 word into a nice to read format.
pub fn format_all_word_bits(word: LC3Word) -> String {
    format_word_bits(word, 0) + " " + &format_word_bits(word, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_255() {
        assert_eq!(format_bits(255), "11111111")
    }

    #[test]
    fn format_0() {
        assert_eq!(format_bits(0), "00000000")
    }

    #[test]
    fn format_128() {
        assert_eq!(format_bits(128), "10000000")
    }

    #[test]
    fn format_full_word() {
        assert_eq!(format_all_word_bits(0xA0B1), "10100000 10110001")
    }
}
