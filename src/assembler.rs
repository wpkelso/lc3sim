use std::collections::{vec_deque, VecDeque};

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

pub fn tokenize(line: &str) -> Result<VecDeque<TokenType>, &str> {
    let mut tokenized_string: VecDeque<TokenType> = VecDeque::new();
    let mut split_string = line.split(";");
    let operation = split_string.next().unwrap().split_whitespace();
    let comment: &str = split_string.next().unwrap();

    for split in operation {
        match split {
            "R0" | "R1" | "R2" | "R3" | "R4" | "R5" | "R6" | "R7" => {
                tokenized_string.push_back(TokenType::REGISTER(RegAddr::One)) // TODO: turn this into a function that returns proper reg addr
            }

            _ => return Err("Unable to assign a token"),
        }
    }

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
