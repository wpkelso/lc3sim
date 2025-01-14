use crate::{
    assembler::{MaybeUnresolvedInstr, Op, PseudoOp, Token},
    defs::{LC3Word, RegAddr},
    instruction::{ADD_OPCODE, AND_OPCODE, ALL_JUMP_OPCODES, BRANCH_OPCODE, JSR_OPCODE, ALL_LOAD_OPCODES, ALL_STORE_OPCODES, TRAP_OPCODE, NOT_OPCODE},
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

/// Second stage of the lexer operation, where a chain of unresolved instructions is created from
/// the asm op. If the line consists only of a comment, then an empty Vec is returned
#[inline]
pub fn construct_instruction_pass(token_chain: &[Token]) -> Result<Vec<MaybeUnresolvedInstr>> {
    let mut result: Vec<MaybeUnresolvedInstr> = Vec::new();

    let operation = &token_chain[0];

    if let Token::INSTR(op) = operation {
        fn check_reg<const SHIFT: usize>(
            token: &Token,
            instr: &mut MaybeUnresolvedInstr,
        ) -> Result<(), anyhow::Error> {
            if let Token::REGISTER(reg) = token {
                instr.value |= (LC3Word::from(*reg) << SHIFT);
                Ok(())
            } else {
                bail!("NOT REG")
            }
        }

        fn check_offset<const SHIFT: u8, const MAX_LEN: u8>(
            token: &Token,
            instr: &mut MaybeUnresolvedInstr,
        ) -> Result<(), anyhow::Error> {
            if let Token::NUM(num) = token {
                let max_mask = const { 1 << (MAX_LEN + 1) };
                if *num < max_mask {
                    instr.value |= num << SHIFT;
                    Ok(())
                } else {
                    bail!("TOO BIG")
                }
            } else if let Token::STRING(label) = token {
                instr
                    .bindings
                    .push((label.clone(), const { SHIFT + MAX_LEN }, SHIFT));
                Ok(())
            } else {
                bail!("NOT OFFSET")
            }
        }

        fn check_reg_or_offset<const SHIFT: u8, const MAX_OFFSET_LEN: u8>(
            token: &Token,
            instr: &mut MaybeUnresolvedInstr,
        ) -> Result<(), anyhow::Error> {
            if let Token::REGISTER(reg) = token {
                instr.value |= (LC3Word::from(*reg) << SHIFT);
                Ok(())
            } else if let Token::NUM(num) = token {
                let max_mask = const { 1 << (MAX_OFFSET_LEN + 1) };
                if *num < max_mask {
                    instr.value |= num << SHIFT;
                    instr.value |= 1 << MAX_OFFSET_LEN;
                    Ok(())
                } else {
                    bail!("TOO BIG")
                }
            } else if let Token::STRING(label) = token {
                instr
                    .bindings
                    .push((label.clone(), const { SHIFT + MAX_OFFSET_LEN }, SHIFT));
                Ok(())
            } else {
                bail!("NOT REG OR OFFSET")
            }
        }

        let (opcode, sequence) = match op {
            Op::ADD => (
                ADD_OPCODE,
                [check_reg::<9>, check_reg::<6>, check_reg_or_offset::<0, 5>].as_slice(),
            ),
            Op::AND => (
                AND_OPCODE, 
                [check_reg::<9>, check_reg::<6>, check_reg_or_offset::<0, 5>].as_slice()),
            Op::LD => (
                ALL_LOAD_OPCODES[0], 
                [check_reg::<9>, check_offset::<0, 9>].as_slice()
            ),
            Op::LDI => (
                ALL_LOAD_OPCODES[1], 
                [check_reg::<9>, check_offset::<0, 9>].as_slice()
            ),
            Op::LDR => (
                ALL_LOAD_OPCODES[2], 
                [check_reg::<9>, check_reg::<6>, check_offset::<0, 6>].as_slice()
            ),
            Op::LEA => (
                ALL_LOAD_OPCODES[3], 
                [check_reg::<9>, check_offset::<0, 9>].as_slice()
            ),
            Op::ST => (
                ALL_STORE_OPCODES[0], 
                [check_reg::<9>, check_offset::<0, 9>].as_slice()
            ),
            Op::STI => (
                ALL_STORE_OPCODES[1], 
                [check_reg::<9>, check_offset::<0, 9>].as_slice()
            ),
            Op::STR => (
                ALL_STORE_OPCODES[2], 
                [check_reg::<9>, check_reg::<6>, check_offset::<0, 6>].as_slice()
            ),
            Op::NOT => (
                NOT_OPCODE,
                [check_reg::<9>, check_reg::<6>].as_slice()
            ),
            _ => todo!(),
        };

        let mut instr = MaybeUnresolvedInstr {
            // Shift opcode to start
            value: (opcode as LC3Word) << 12,
            bindings: Vec::new(),
        };

        for (process, token) in sequence.iter().zip(&token_chain[1..]) {
            process(token, &mut instr)?;
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
            Token::REGISTER(RegAddr::Zero)
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0101000001000000);

        let test_vec = vec![
            Token::INSTR(Op::AND),
            Token::REGISTER(RegAddr::Three),
            Token::REGISTER(RegAddr::One),
            Token::NUM(0b10011)
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
            Token::REGISTER(RegAddr::Zero)
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0001000001000000);

        let test_vec = vec![
            Token::INSTR(Op::ADD),
            Token::REGISTER(RegAddr::Three),
            Token::REGISTER(RegAddr::One),
            Token::NUM(0b10011)
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
            Token::NUM(0b000111000)
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0010101000111000);

        let test_vec = vec![
            Token::INSTR(Op::LDI),
            Token::REGISTER(RegAddr::Five),
            Token::NUM(0b000111000)
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1010101000111000);

        let test_vec = vec![
            Token::INSTR(Op::LDR),
            Token::REGISTER(RegAddr::Five),
            Token::REGISTER(RegAddr::Two),
            Token::NUM(0b111000)
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0110101010111000);

        let test_vec = vec![
            Token::INSTR(Op::LEA),
            Token::REGISTER(RegAddr::Five),
            Token::NUM(0b000111000)
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
            Token::NUM(0b000111000)
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0011101000111000);

        let test_vec = vec![
            Token::INSTR(Op::STI),
            Token::REGISTER(RegAddr::Five),
            Token::NUM(0b000111000)
        ];
        let (label, instr) = lexer(&test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1011101000111000);

        let test_vec = vec![
            Token::INSTR(Op::STR),
            Token::REGISTER(RegAddr::Five),
            Token::REGISTER(RegAddr::Two),
            Token::NUM(0b111000)
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
        // This is the value that should be produced. Currently this fails, as there is no way to
        // insert arbitrary bits into instructions when forming them.
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1001101000111111);
    }
}
