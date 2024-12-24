use once_cell::sync::Lazy;
use regex::{bytes::RegexSet, Regex};
use std::collections::VecDeque;

use crate::defs::{LC3Word, Op, PseudoOp, RegAddr};

pub struct MaybeUnresolvedInstr {
    value: LC3Word,
    ///Label, Start offset, End offset
    bindings: Option<(String, u8, u8)>,
}

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

const META_PATTERN: [&str; 5] = [r".ORIG", r".FILL", r"BLKW", r".STRINGZ", r".END"];
const NUM_PATTERN: &str = r"[x|#|b]-?[0-9A-F]*";
const REG_PATTERN: &str = r"R[0-7]";
const COMMENT_PATTERN: &str = r";*";
const LABEL_PATTERN: &str = r"[0-9a-zA-Z]+";

pub fn tokenize(line: &str) -> Result<VecDeque<TokenType>, &str> {
    // Regexes get lazy compiled then stored for reuse
    static RE_REGISTER: Lazy<Regex> = Lazy::new(|| Regex::new(REG_PATTERN).unwrap());
    static RE_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(COMMENT_PATTERN).unwrap());
    static RE_INSTR: Lazy<RegexSet> = Lazy::new(|| RegexSet::new(INSTR_PATTERN).unwrap());
    static RE_META: Lazy<RegexSet> = Lazy::new(|| RegexSet::new(META_PATTERN).unwrap());
    static RE_NUM: Lazy<Regex> = Lazy::new(|| Regex::new(NUM_PATTERN).unwrap());
    static RE_LABEL: Lazy<Regex> = Lazy::new(|| Regex::new(LABEL_PATTERN).unwrap());

    let mut tokenized_string: VecDeque<TokenType> = VecDeque::new();
    Ok(tokenized_string)
}

pub fn translate_line(line: &str) -> MaybeUnresolvedInstr {
    todo!()
}

pub fn resolve_instr(instr: MaybeUnresolvedInstr) -> String {
    todo!()
}
