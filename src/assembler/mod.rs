use crate::defs::{LC3Word, Op, PseudoOp, RegAddr};
use strum::EnumIs;
use strum_macros::EnumDiscriminants;

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

pub fn translate_line(line: &str) -> MaybeUnresolvedInstr {
    todo!()
}

pub fn resolve_instr(instr: MaybeUnresolvedInstr) -> String {
    todo!()
}
