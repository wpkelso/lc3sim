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

/// Second stage of the lexer operation, where a chain of unresolved instructions is created from
/// the asm op. If the line consists only of a comment, then an empty Vec is returned
#[inline]
pub fn construct_instruction_pass(token_chain: &[Token]) -> Result<Vec<MaybeUnresolvedInstr>> {
    let result:Vec<MaybeUnresolvedInstr> = Vec::new(); 

    let operation = token_chain[0].clone();

    if operation.is_instr() || operation.is_meta() {
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
        let test_vec = vec![Token::STRING("LABEL1".to_string()), Token::INSTR(Op::ILLEGAL)];
        let (label, instr) = prefix_label_pass(&test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr[0], Token::INSTR(Op::ILLEGAL));
    }
}
