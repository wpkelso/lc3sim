use crate::defs::{LC3Word, Op, PseudoOp, RegAddr};
use strum::EnumIs;
use strum_macros::EnumDiscriminants;
use anyhow::Result;

pub mod lexer;
pub mod tokenizer;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MaybeUnresolvedInstr {
    value: LC3Word,
    ///Label, Start offset, End offset
    bindings: Vec<(String, u8, u8)>,
}

#[derive(Debug, Clone, Eq, PartialEq, EnumIs, EnumDiscriminants)]
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

pub fn translate_line(line: &str) -> Result<Vec<MaybeUnresolvedInstr>> {
    let (instruction, comment) = line.split_once(';').unwrap();
    let splits = instruction.split_ascii_whitespace();
    let mut token_chain: Vec<Token> = Vec::new();

    for split in splits {
        let mut tokens = tokenizer::tokenize(split)?;
        token_chain.append(&mut tokens);
    }
    
    let (label, chain) = lexer::lexer(&token_chain);

    //TODO: add label to symbol table
    chain
}

pub fn resolve_instr(instr: MaybeUnresolvedInstr) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn translate_instr() {
        let instruction: &str = "AND R0, R1, R0;";
        let machine_code = translate_line(instruction).unwrap();

        assert_eq!(machine_code.len(), 1);
        assert_eq!(machine_code.first().unwrap().value, 0b0101000001000000);
    }
}
