use crate::{
    assembler::{MaybeUnresolvedInstr, Op, PseudoOp, Token},
    defs::LC3Word,
    instruction::{ADD_OPCODE, AND_OPCODE},
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
    let result: Vec<MaybeUnresolvedInstr> = Vec::new();

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
            Op::AND => (AND_OPCODE, [check_reg::<9>, check_reg::<6>].as_slice()),
            _ => todo!(),
        };

        let mut instr = MaybeUnresolvedInstr {
            // Shift opcode to start
            value: (opcode as LC3Word) << 12,
            bindings: Vec::new(),
        };

        for (process, token) in sequence.into_iter().zip(&token_chain[1..]) {
            process(token, &mut instr)?;
        }
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
}
