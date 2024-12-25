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
    r"^ADD$",
    r"^AND$",
    r"^BR[nN]?[zZ]?[pP]?$",
    r"^JMP$",
    r"^JSR$",
    r"^JSRR$",
    r"^LD$",
    r"^LDI$",
    r"^LDR$",
    r"^LEA$",
    r"^NOT$",
    r"^RET$",
    r"^RTI$",
    r"^ST$",
    r"^STI$",
    r"^STR$",
    r"^TRAP$",
    r"^GETC$",
    r"^OUT$",
    r"^PUTS$",
    r"^IN$",
    r"^PUTSP$",
    r"^HALT$",
];

const META_PATTERN: [&str; 5] = [r"^.ORIG$", r"^.FILL$", r"^BLKW$", r"^.STRINGZ$", r"^.END$"];
const NUM_PATTERN: &str = r"^[x|#|b]-?[0-9A-F]*$";
const REG_PATTERN: &str = r"^R[0-7]$";
const COMMENT_PATTERN: &str = r"^;.*$";
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
        let reg_num_char: char = line.chars().nth(1).unwrap();
        let reg_num_int: u8 = reg_num_char.to_digit(10).unwrap() as u8;
        token = TokenType::REGISTER(RegAddr::try_from(reg_num_int).unwrap());
        Ok(token)
    } else if RE_COMMENT.is_match(line) {
        token = TokenType::COMMENT(line.to_string());
        Ok(token)
    } else if RE_INSTR.is_match(line.as_bytes()) {
        let matches: Vec<_> = RE_INSTR.matches(line.as_bytes()).into_iter().collect();
        let mut instr_type: Op = Op::ILLEGAL;
        for item in matches {
            // this should be fine because there should only ever be 1 item in the vec
            // if there isn't then we just match the last
            instr_type = match item {
                0 => Op::ADD,
                1 => Op::AND,
                2 => {
                    let n: bool = line.contains(['n', 'N']);
                    let z: bool = line.contains(['z', 'Z']);
                    let p: bool = line.contains(['p', 'P']);
                    Op::BR(n, z, p)
                }
                3 => Op::JMP,
                4 => Op::JSR,
                5 => Op::JSRR,
                6 => Op::LD,
                7 => Op::LDI,
                8 => Op::LDR,
                9 => Op::LEA,
                10 => Op::RET,
                11 => Op::RTI,
                12 => Op::ST,
                13 => Op::STI,
                14 => Op::STR,
                15 => Op::TRAP,
                16 => Op::GETC,
                17 => Op::OUT,
                18 => Op::PUTS,
                19 => Op::IN,
                20 => Op::PUTSP,
                21 => Op::HALT,
                _ => return Err("Couldn't match a legal instruction"),
            };
        }
        token = TokenType::INSTR(instr_type);
        Ok(token)
    } else if RE_META.is_match(line.as_bytes()) {
        let matches: Vec<_> = RE_META.matches(line.as_bytes()).into_iter().collect();
        let mut pseudo_instr_type: PseudoOp = PseudoOp::ILLEGAL;

        for item in matches {
            // this should be fine as there should only ever be 1 item in the vec
            // if there isn't then we just match the last
            pseudo_instr_type = match item {
                0 => PseudoOp::ORIG,
                1 => PseudoOp::FILL,
                2 => PseudoOp::BLKW,
                3 => PseudoOp::STRINGZ,
                4 => PseudoOp::END,
                _ => return Err("Couldn't match a legal pseudo-instruction"),
            };
        }
        token = TokenType::META(pseudo_instr_type);
        Ok(token)
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
    use std::result;

    use super::*;

    #[test]
    fn tokenize_register() {
        // The number of register is small enough that checking that all of them parse manually is fine
        let test_str: &str = "R0";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::REGISTER(RegAddr::Zero));

        let test_str: &str = "R1";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::REGISTER(RegAddr::One));

        let test_str: &str = "R2";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::REGISTER(RegAddr::Two));

        let test_str: &str = "R3";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::REGISTER(RegAddr::Three));

        let test_str: &str = "R4";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::REGISTER(RegAddr::Four));

        let test_str: &str = "R5";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::REGISTER(RegAddr::Five));

        let test_str: &str = "R6";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::REGISTER(RegAddr::Six));

        let test_str: &str = "R7";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::REGISTER(RegAddr::Seven));
    }

    #[test]
    #[should_panic]
    fn tokenize_unclean_register() {
        let test_str: &str = "R0, A_LABEL";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_ne!(result, TokenType::REGISTER(RegAddr::Zero));
    }

    #[test]
    fn tokenize_comment() {
        let test_str: &str = "; This is a test comment";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(
            result,
            TokenType::COMMENT("; This is a test comment".to_string())
        );
    }

    #[test]
    fn tokenize_instr_add() {
        let test_str: &str = "ADD";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::INSTR(Op::ADD));
    }

    #[test]
    fn tokenize_meta_orig() {
        let test_str: &str = ".ORIG";
        let result: TokenType = tokenize(test_str).unwrap();
        assert_eq!(result, TokenType::META(PseudoOp::ORIG));
    }
}
