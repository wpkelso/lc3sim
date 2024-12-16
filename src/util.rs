use crate::defs::LC3Word;

/// Format bits from a single byte into a nice to read format.
pub fn format_bits(mut byte: u8) -> String {
    let mut dest = ['0'; 4];

    if byte > 7 {
        byte -= 8;
        dest[0] = '1';
    }
    if byte > 3 {
        byte -= 4;
        dest[1] = '1';
    }
    if byte > 1 {
        byte -= 2;
        dest[2] = '1';
    }
    if byte != 0 {
        dest[3] = '1';
    }

    dest.into_iter().collect()
}

/// Format bits from `idx` byte of a LC3 word into a nice to read format.
pub fn format_word_bits(word: LC3Word, idx: usize) -> String {
    format_bits(word.to_be_bytes()[idx])
}
