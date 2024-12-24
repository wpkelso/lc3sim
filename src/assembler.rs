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

const INSTR_PATTERNS: [&str; 23] = [
    "ADD",
    "AND",
    "BR[nzpNZP]*",
    "JMP",
    "JSR",
    "JSRR",
    "LD",
    "LDI",
    "LDR",
    "LEA",
    "NOT",
    "RET",
    "RTI",
    "ST",
    "STI",
    "STR",
    "TRAP",
    "GETC",
    "OUT",
    "PUTS",
    "IN",
    "PUTSP",
    "HALT",
];

const META_PATTERNS: [&str; 5] = [".ORIG", ".FILL", "BLKW", ".STRINGZ", ".END"];

pub fn tokenize(line: &str) -> Result<VecDeque<TokenType>, &str> {
    // Regexes get lazy compiled then stored for reuse
    static RE_REGISTER: Lazy<Regex> = Lazy::new(|| Regex::new(r"R[0-7]").unwrap());
    static RE_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r";.*").unwrap());
    static RE_INSTR: Lazy<RegexSet> = Lazy::new(|| RegexSet::new(INSTR_PATTERNS).unwrap());
    static RE_META: Lazy<RegexSet> = Lazy::new(|| RegexSet::new(META_PATTERNS).unwrap());
    static RE_NUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"[x|#|b]-?[0-9A-F]*").unwrap());

    let mut tokenized_string: VecDeque<TokenType> = VecDeque::new();
    let operation = split_string.next().unwrap().split_whitespace();
    let comment: &str = split_string.next().unwrap();

    for split in operation {}

    Ok(tokenized_string)
}

pub fn translate_line(line: &str) -> MaybeUnresolvedInstr {
    // 1st element: either be a LABEL or a INSTR
    // 2nd element: INSTR if LABEL in 1 else PARAMETER
    // last element might also be LABEL

    MaybeUnresolvedInstr {
        value: 0x0,
        bindings: None,
    }
}

pub fn resolve_instr(instr: MaybeUnresolvedInstr) -> String {
    todo!()
}
