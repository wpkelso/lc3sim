use once_cell::sync::Lazy;
use regex::{bytes::RegexSet, Regex};

use crate::defs::{LC3Word, Op, PseudoOp, RegAddr};

pub struct MaybeUnresolvedInstr {
    value: LC3Word,
    ///Label, Start offset, End offset
    bindings: Option<(String, u8, u8)>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    INSTR(Op),
    REGISTER(RegAddr),
    META(PseudoOp),
    STRING(String),
    NUM(LC3Word),
    COMMENT(String),
    QUOTES,
    SEMICOLON,
    COMMA,
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
const STRING_PATTERN: &str = r"^[0-9a-zA-Z[:punct:]]+$";

/// Take in a `&str`, returning a `Vec<Token>` that contains all syntax morphemes in the str.
pub fn tokenize(line: &str) -> Result<Vec<Token>, &str> {
    // Regexes get lazy compiled then stored for reuse
    static RE_REGISTER: Lazy<Regex> = Lazy::new(|| Regex::new(REG_PATTERN).unwrap());
    static RE_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(COMMENT_PATTERN).unwrap());
    static RE_INSTR: Lazy<RegexSet> = Lazy::new(|| RegexSet::new(INSTR_PATTERN).unwrap());
    static RE_META: Lazy<RegexSet> = Lazy::new(|| RegexSet::new(META_PATTERN).unwrap());
    static RE_NUM: Lazy<Regex> = Lazy::new(|| Regex::new(NUM_PATTERN).unwrap());
    static RE_STRING: Lazy<Regex> = Lazy::new(|| Regex::new(STRING_PATTERN).unwrap());

    let mut token: Vec<Token> = Vec::new(); // this value is ultimately returned

    if RE_REGISTER.is_match(line) {
        let reg_num_char: char = line.chars().nth(1).unwrap();
        let reg_num_int: u8 = reg_num_char.to_digit(10).unwrap() as u8;
        token.push(Token::REGISTER(RegAddr::try_from(reg_num_int).unwrap()));
        Ok(token)
    } else if RE_COMMENT.is_match(line) {
        token.push(Token::COMMENT(line.to_string()));
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
                    // this was written before this returned a vector of tokens
                    // it might be better to turn these into separate tokens
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
        token.push(Token::INSTR(instr_type));
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
        token.push(Token::META(pseudo_instr_type));
        Ok(token)
    } else if RE_NUM.is_match(line) {
        todo!()
    } else if RE_STRING.is_match(line.trim_matches('"')) {
        // Strings and labels are functionally the same but one has quotes.
        // Therefore they aren't differentiated here, and should be dealt with
        // during lexing
        let string = line.trim_matches('"').to_string();
        if line.starts_with('"') {
            token.push(Token::QUOTES)
        }
        token.push(Token::STRING(string));
        if line.ends_with('"') {
            token.push(Token::QUOTES)
        }
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
    use super::*;

    #[test]
    fn tokenize_register() {
        // The number of registers is small enough that checking that all of them parse manually is fine
        let test_str: &str = "R0";
        let result: Vec<Token> = tokenize(test_str).unwrap();
        assert_eq!(result[0], Token::REGISTER(RegAddr::Zero));
    }

    #[test]
    #[should_panic]
    fn tokenize_unclean_register() {
        let test_str: &str = "R0, A_LABEL";
        let result: Vec<Token> = tokenize(test_str).unwrap();
        assert_ne!(result[0], Token::REGISTER(RegAddr::Zero));
    }

    #[test]
    fn tokenize_comment() {
        let test_str: &str = "; This is a test comment";
        let result: Vec<Token> = tokenize(test_str).unwrap();
        assert_eq!(
            result[0],
            Token::COMMENT("; This is a test comment".to_string())
        );
    }

    #[test]
    fn tokenize_instr_add() {
        let test_str: &str = "ADD";
        let result: Vec<Token> = tokenize(test_str).unwrap();
        assert_eq!(result[0], Token::INSTR(Op::ADD));
    }

    #[test]
    fn tokenize_meta_orig() {
        let test_str: &str = ".ORIG";
        let result: Vec<Token> = tokenize(test_str).unwrap();
        assert_eq!(result[0], Token::META(PseudoOp::ORIG));
    }

    #[test]
    fn tokenize_string_and_label() {
        let test_str: &str = "Strings!";
        let result: Vec<Token> = tokenize(test_str).unwrap();
        assert_eq!(result[0], Token::STRING("Strings!".to_string()));
    }
}
