use std::mem::discriminant;
use crate::assembler::{MaybeUnresolvedInstr, Op, PseudoOp, Token};
use anyhow::{Result, bail};

// All of these functions are inlined because they work on the same exact data but are split up for
// legibility

/// First stage of the lexer operation, where any prefix labels are stripped out 
#[inline]
pub fn prefix_label_pass(token_chain: &[Token])  -> (Option<&str>, &[Token]) {
    // we create a shell STRING variant to get it's discriminant, as this is more flexible than if
    // we hardcode the discriminant value
    let label_discriminant = discriminant(&Token::STRING("".to_string()));
    let target_discriminant = discriminant(&token_chain[0]);

    if target_discriminant.eq(&label_discriminant) {
        let label_str: &str = match &token_chain[0] { Token::STRING(label) => label.as_str(), _ => panic!("This shouldn't happen")};
        (Some(label_str), &token_chain[1..])
    } else {
        (None, token_chain)
    }
}

#[inline]
/// Validates that a chain consists of a valid sequence of tokens for a given instruction
/// (irrespective of any value they may have), returning a boolean value to reflect that 
/// validation. Will return an Err value if fed an invalid instruction.
pub fn validate_instruction(operation: Op, token_chain: &[Token]) -> Result<bool> {
    // return a false value if a sequence cannot be validated completely
    let mut validation: bool = false;

    match operation {
        Op::ADD => (),
        Op::AND => (),
        Op::BR(_, _, _) => (),
        Op::JMP => (),
        Op::JSR => (),
        Op::JSRR => (),
        Op::LD => (),
        Op::LDI => (),
        Op::LDR => (),
        Op::LEA => (),
        Op::NOT => (),
        Op::RET => (),
        Op::RTI => (),
        Op::ST => (),
        Op::STI => (),
        Op::STR => (),
        Op::TRAP => (),
        Op::GETC => (),
        Op::OUT => (),
        Op::PUTS => (),
        Op::IN => (),
        Op::PUTSP => (),
        Op::HALT => (),
        // Covers invalid or illegal instructions that were provided
        _ => bail!("Unknown instruction, cannot validate.")

    }

    Ok(validation)
}

#[inline]
pub fn validate_pseudo_instruction(operation: PseudoOp, token_chain: &[Token]) {

}

/// Second stage of the lexer operation, where a chain of unresolved instructions is created from
/// the asm op. If the line consists only of a comment, then an empty Vec is returned
#[inline]
pub fn construct_instruction_pass(token_chain: &[Token]) -> Result<Vec<MaybeUnresolvedInstr>> {
    let result:Vec<MaybeUnresolvedInstr> = Vec::new(); 

    match token_chain[0] {
        // Validate that you can form a valid instruction first, then transform it into an
        // instruction in a second pass
        Token::INSTR(_) => print!(""),
        Token::META(_) => print!(""),
        _ => bail!("Line is invalid, does not start with an instruction!"),
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
        let test_vec = vec![Token::STRING("LABEL1".to_string()), Token::INSTR(Op::ILLEGAL)];
        let (label, instr) = prefix_label_pass(&test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr[0], Token::INSTR(Op::ILLEGAL));
    }
}
