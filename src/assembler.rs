use once_cell::sync::Lazy;
use regex::{bytes::RegexSet, Regex};

use crate::defs::{LC3Word, Op, PseudoOp, RegAddr};

pub struct MaybeUnresolvedInstr {
    value: LC3Word,
    ///Label, Start offset, End offset
    bindings: Option<(String, u8, u8)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LABEL(String),
    INSTR(Op),
    REGISTER(RegAddr),
    META(PseudoOp),
    STRING(String),
    NUM(LC3Word),
    COMMENT(String),
}

// This follows the same ordering as defs.rs > pub enum Op
const INSTR_PATTERN: [&str; 23] = [
    r"ADD",
    r"AND",
    r"BR[nzpNZP]*",
    r"JMP",
    r"JSR",
    r"JSRR",
    r"LD",
    r"LDI",
    r"LDR",
    r"LEA",
    r"NOT",
    r"RET",
    r"RTI",
    r"ST",
    r"STI",
    r"STR",
    r"TRAP",
    r"GETC",
    r"OUT",
    r"PUTS",
    r"IN",
    r"PUTSP",
    r"HALT",
];

const META_PATTERN: [&str; 5] = [r"^.ORIG$", r"^.FILL$", r"^BLKW$", r"^.STRINGZ$", r"^.END$"];
const NUM_PATTERN: &str = r"^[x|#|b]-?[0-9A-F]*$";
const REG_PATTERN: &str = r"^R[0-7]$";
const COMMENT_PATTERN: &str = r"^;*$";
const LABEL_PATTERN: &str = r"^[0-9a-zA-Z]+$";

pub fn tokenize(line: &str) -> Result<TokenType, &str> {
    // Regexes get lazy compiled then stored for reuse
    static RE_REGISTER: Lazy<Regex> = Lazy::new(|| Regex::new(REG_PATTERN).unwrap());
    static RE_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(COMMENT_PATTERN).unwrap());
    static RE_INSTR: Lazy<RegexSet> = Lazy::new(|| RegexSet::new(INSTR_PATTERN).unwrap());
    static RE_META: Lazy<RegexSet> = Lazy::new(|| RegexSet::new(META_PATTERN).unwrap());
    static RE_NUM: Lazy<Regex> = Lazy::new(|| Regex::new(NUM_PATTERN).unwrap());
    static RE_LABEL: Lazy<Regex> = Lazy::new(|| Regex::new(LABEL_PATTERN).unwrap());

    let token: TokenType;

    if RE_REGISTER.is_match(line) {
        let reg_num: u8 = *line.as_bytes().get(line.len()).unwrap();
        token = TokenType::REGISTER(RegAddr::try_from(reg_num).unwrap());
        return Ok(token);
    } else if RE_COMMENT.is_match(line) {
        token = TokenType::COMMENT(line.to_string());
        return Ok(token);
    } else {
        return Err("Couldn't form token");
    }
}

pub fn translate_line(line: &str) -> MaybeUnresolvedInstr {
    todo!()
}

pub fn resolve_instr(instr: MaybeUnresolvedInstr) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tokenize_register() {
        let test_str: &str = "R1";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::REGISTER(RegAddr::One));
    }
}
