use crate::{
    assembler::{MaybeUnresolvedInstr, Op, PseudoOp, Token},
    defs::{LC3Word, RegAddr},
    instruction::{
        ADD_OPCODE, ALL_JUMP_OPCODES, ALL_LOAD_OPCODES, ALL_STORE_OPCODES, AND_OPCODE,
        BRANCH_OPCODE, JSR_OPCODE, NOT_OPCODE, RET_OPCODE, RTI_OPCODE, TRAP_OPCODE,
    },
};
use anyhow::{bail, Result};

// All of these functions are inlined because they work on the same exact data but are split up for
// legibility

/// First stage of the lexer operation, where any prefix labels are stripped out
#[inline]
pub fn prefix_label_pass(token_chain: &[Token]) -> (Option<&str>, &[Token]) {
    if token_chain[0].is_string() {
        let label_str: &str = match &token_chain[0] {
            Token::STRING(label) => label.as_str(),
            _ => panic!("This shouldn't happen"),
        };
        (Some(label_str), &token_chain[1..])
    } else {
        (None, token_chain)
    }
}

/// Validate whether the given token is a register, and overlay it onto the given machine instruction
fn check_reg(token: &Token, shift: usize) -> Result<LC3Word, anyhow::Error> {
    let mut value: LC3Word = 0b0;
    if let Token::REGISTER(reg) = token {
        value |= LC3Word::from(*reg) << shift;
    } else {
        bail!("NOT REG")
    }

    Ok(value)
}

/// Validate whether the given token is an offset, and overlay it onto the given machine instruction
fn check_offset(token: &Token, shift: u8, max_len: u8) -> Result<LC3Word, anyhow::Error> {
    let mut value: LC3Word = 0b0;

    if let Token::NUM(num) = token {
        let max_mask = 1 << (max_len + 1);
        if *num < max_mask {
            value |= num << shift;
        } else {
            bail!("TOO BIG")
        }
    } else if let Token::STRING(label) = token {
        /*instr.bindings.push((label.clone(), shift + max_len, shift));*/
    } else {
        bail!("NOT OFFSET")
    }

    Ok(value)
}

/// Validate whether the given token is an offset OR a register, and overlay it on the given machine instruction
fn check_reg_or_offset(
    token: &Token,
    shift: u8,
    max_offset_len: u8,
) -> Result<LC3Word, anyhow::Error> {
    let mut value: LC3Word = 0b0;

    if let Token::REGISTER(reg) = token {
        value |= LC3Word::from(*reg) << shift;
    } else if let Token::NUM(num) = token {
        let max_mask = 1 << (max_offset_len + 1);
        if *num < max_mask {
            value |= num << shift;
            value |= 1 << max_offset_len;
        } else {
            bail!("TOO BIG")
        }
    } else if let Token::STRING(label) = token {
        /*instr
        .bindings
        .push((label.clone(), shift + max_offset_len, shift));*/
    } else {
        bail!("NOT REG OR OFFSET")
    }

    Ok(value)
}

fn translate_to_machine_code(operation: &Op, chain: &[Token]) -> (u8, Vec<LC3Word>) {
    let mut token = chain[1..].iter();

    match operation {
        Op::ADD => (
            ADD_OPCODE,
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_reg(token.next().unwrap(), 6).unwrap(),
                check_reg_or_offset(token.next().unwrap(), 0, 5).unwrap(),
            ],
        ),
        Op::AND => (
            AND_OPCODE,
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_reg(token.next().unwrap(), 6).unwrap(),
                check_reg_or_offset(token.next().unwrap(), 0, 5).unwrap(),
            ],
        ),
        Op::LD => (
            ALL_LOAD_OPCODES[0],
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_offset(token.next().unwrap(), 0, 9).unwrap(),
            ],
        ),
        Op::LDI => (
            ALL_LOAD_OPCODES[1],
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_offset(token.next().unwrap(), 0, 9).unwrap(),
            ],
        ),
        Op::LDR => (
            ALL_LOAD_OPCODES[2],
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_reg(token.next().unwrap(), 6).unwrap(),
                check_offset(token.next().unwrap(), 0, 6).unwrap(),
            ],
        ),
        Op::LEA => (
            ALL_LOAD_OPCODES[3],
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_offset(token.next().unwrap(), 0, 9).unwrap(),
            ],
        ),
        Op::ST => (
            ALL_STORE_OPCODES[0],
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_offset(token.next().unwrap(), 0, 9).unwrap(),
            ],
        ),
        Op::STI => (
            ALL_STORE_OPCODES[1],
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_offset(token.next().unwrap(), 0, 9).unwrap(),
            ],
        ),
        Op::STR => (
            ALL_STORE_OPCODES[2],
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_reg(token.next().unwrap(), 6).unwrap(),
                check_offset(token.next().unwrap(), 0, 6).unwrap(),
            ],
        ),
        Op::NOT => (
            NOT_OPCODE,
            vec![
                check_reg(token.next().unwrap(), 9).unwrap(),
                check_reg(token.next().unwrap(), 6).unwrap(),
                0b111111,
            ],
        ),
        Op::JMP => (
            ALL_JUMP_OPCODES[0],
            vec![check_reg(token.next().unwrap(), 6).unwrap()],
        ),
        Op::RET => (RET_OPCODE, vec![0b111000000]),
        Op::RTI => (RTI_OPCODE, vec![0b0]),

        _ => todo!(),
    }
}

/// Second stage of the lexer operation, where a chain of unresolved instructions is created from
/// the asm op. If the line consists only of a comment, then an empty Vec is returned
#[inline]
pub fn construct_instruction_pass(token_chain: &[Token]) -> Result<Vec<MaybeUnresolvedInstr>> {
    let mut result: Vec<MaybeUnresolvedInstr> = Vec::new();

    let operation = &token_chain[0];

    if let Token::INSTR(op) = operation {
        let (opcode, values) = translate_to_machine_code(op, token_chain);

        let mut instr = MaybeUnresolvedInstr {
            value: (opcode as LC3Word) << 12,
            bindings: Vec::new(),
        };

        for val in values {
            instr.value |= val;
        }

        result.push(instr);
    } else if operation.is_meta() {
        todo!()
    } else if !operation.is_comment() {
        bail!("Line is invalid, does not start with an instruction!")
    }

    Ok(result)
}

/// Wrapper function to provide a cleaner API for the lexing passes
pub fn lexer(token_chain: &[Token]) -> (Option<&str>, Result<Vec<MaybeUnresolvedInstr>>) {
    let (label, chain) = prefix_label_pass(token_chain);
    let result = construct_instruction_pass(chain);

    // The result gets passed on so the assembler can attatch more context to any error messages
    // generated (i.e. the expected address of the error)
    (label, result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lex_label_instr() {
        let test_vec = vec![
            Token::STRING("LABEL1".to_string()),
            Token::INSTR(Op::ILLEGAL),
        ];
        let (label, instr) = prefix_label_pass(&test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr[0], Token::INSTR(Op::ILLEGAL));
    }

    #[test]
    fn lex_and_instr() {
        let test_vec = vec![
            Token::STRING("LABEL1".to_string()),
            Token::INSTR(Op::AND),
            Token::REGISTER(RegAddr::Zero),
            Token::REGISTER(RegAddr::One),
            Token::REGISTER(RegAddr::Zero),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0101000001000000);

        let test_vec = vec![
            Token::INSTR(Op::AND),
            Token::REGISTER(RegAddr::Three),
            Token::REGISTER(RegAddr::One),
            Token::NUM(0b10011),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0101011001110011);
    }

    #[test]
    fn lex_add_instr() {
        let test_vec = vec![
            Token::STRING("LABEL1".to_string()),
            Token::INSTR(Op::ADD),
            Token::REGISTER(RegAddr::Zero),
            Token::REGISTER(RegAddr::One),
            Token::REGISTER(RegAddr::Zero),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0001000001000000);

        let test_vec = vec![
            Token::INSTR(Op::ADD),
            Token::REGISTER(RegAddr::Three),
            Token::REGISTER(RegAddr::One),
            Token::NUM(0b10011),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0001011001110011);
    }

    #[test]
    fn lex_load_instrs() {
        let test_vec = vec![
            Token::INSTR(Op::LD),
            Token::REGISTER(RegAddr::Five),
            Token::NUM(0b000111000),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0010101000111000);

        let test_vec = vec![
            Token::INSTR(Op::LDI),
            Token::REGISTER(RegAddr::Five),
            Token::NUM(0b000111000),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1010101000111000);

        let test_vec = vec![
            Token::INSTR(Op::LDR),
            Token::REGISTER(RegAddr::Five),
            Token::REGISTER(RegAddr::Two),
            Token::NUM(0b111000),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0110101010111000);

        let test_vec = vec![
            Token::INSTR(Op::LEA),
            Token::REGISTER(RegAddr::Five),
            Token::NUM(0b000111000),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1110101000111000);
    }

    #[test]
    fn lex_store_instrs() {
        let test_vec = vec![
            Token::INSTR(Op::ST),
            Token::REGISTER(RegAddr::Five),
            Token::NUM(0b000111000),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0011101000111000);

        let test_vec = vec![
            Token::INSTR(Op::STI),
            Token::REGISTER(RegAddr::Five),
            Token::NUM(0b000111000),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1011101000111000);

        let test_vec = vec![
            Token::INSTR(Op::STR),
            Token::REGISTER(RegAddr::Five),
            Token::REGISTER(RegAddr::Two),
            Token::NUM(0b111000),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0111101010111000);
    }

    #[test]
    fn lex_not_instr() {
        let test_vec = vec![
            Token::INSTR(Op::NOT),
            Token::REGISTER(RegAddr::Five),
            Token::REGISTER(RegAddr::Zero),
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1001101000111111);
    }

    #[test]
    fn lex_return_instr() {
        let test_vec = vec![Token::INSTR(Op::RET)];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1100000111000000);

        let test_vec = vec![Token::INSTR(Op::RTI)];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1000000000000000);
    }
}
