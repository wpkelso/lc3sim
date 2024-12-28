use std::mem::discriminant;
use crate::assembler::{MaybeUnresolvedInstr, Op, PseudoOp, Token};
use anyhow::Result;

/// First stage of the lexer operation, where any prefix labels are stripped out 
pub fn prefix_label_pass(token_chain: &[Token])  -> (Option<&str>, &[Token]){
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
pub fn construct_instruction_pass(token_chain: Vec<Token>) -> Result<Vec<MaybeUnresolvedInstr>> {
    todo!();
}
