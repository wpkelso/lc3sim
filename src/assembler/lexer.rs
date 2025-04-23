use crate::{
    assembler::{MaybeUnresolvedInstr, Op, PseudoOp, Token},
    defs::{LC3Word, RegAddr},
};
use anyhow::{bail, Result};

// All of these functions are inlined because they work on the same exact data but are split up for
// legibility

/// First stage of the lexer operation, where any prefix labels are stripped out
#[inline]
pub fn prefix_label_pass(token_chain: Vec<Token>) -> (Option<String>, Vec<Token>) {
    if token_chain[0].is_string() {
        let label_str: String = match token_chain[0].clone() {
            Token::STRING(label) => label,
            _ => panic!("This shouldn't happen"),
        };

        let (_, token_chain) = token_chain.split_at(1);
        (Some(label_str), Vec::from(token_chain))
    } else {
        (None, token_chain)
    }
}

/// Second stage of the lexer operation, where a chain of unresolved instructions is created from
/// the asm op. If the line consists only of a comment, then an empty Vec is returned
#[inline]
pub fn construct_instruction_pass(token_chain: Vec<Token>) -> Result<Vec<MaybeUnresolvedInstr>> {
    let mut result: Vec<MaybeUnresolvedInstr> = Vec::new();

    result.push(MaybeUnresolvedInstr::new_from_chain(token_chain));

    Ok(result)
}

/// Wrapper function to provide a cleaner API for the lexing passes
pub fn lexer(token_chain: Vec<Token>) -> (Option<String>, Result<Vec<MaybeUnresolvedInstr>>) {
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
        let (label, instr) = prefix_label_pass(test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr[0], Token::INSTR(Op::ILLEGAL));
    }

    #[test]
    fn lex_and_instr() {
        let test_vec = vec![
            Token::STRING("LABEL1".to_string()),
            Token::INSTR(Op::AND),
            Token::REGISTER(RegAddr::Zero),
            Token::COMMA,
            Token::REGISTER(RegAddr::One),
            Token::COMMA,
            Token::REGISTER(RegAddr::Zero),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0101000001000000);

        let test_vec = vec![
            Token::INSTR(Op::AND),
            Token::REGISTER(RegAddr::Three),
            Token::COMMA,
            Token::REGISTER(RegAddr::One),
            Token::COMMA,
            Token::NUM(0b10011),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0101011001110011);
    }

    #[test]
    fn lex_add_instr() {
        let test_vec = vec![
            Token::STRING("LABEL1".to_string()),
            Token::INSTR(Op::ADD),
            Token::REGISTER(RegAddr::Zero),
            Token::COMMA,
            Token::REGISTER(RegAddr::One),
            Token::COMMA,
            Token::REGISTER(RegAddr::Zero),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label.unwrap(), "LABEL1");
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0001000001000000);

        let test_vec = vec![
            Token::INSTR(Op::ADD),
            Token::REGISTER(RegAddr::Three),
            Token::COMMA,
            Token::REGISTER(RegAddr::One),
            Token::COMMA,
            Token::NUM(0b10011),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0001011001110011);
    }

    #[test]
    fn lex_load_instrs() {
        let test_vec = vec![
            Token::INSTR(Op::LD),
            Token::REGISTER(RegAddr::Five),
            Token::COMMA,
            Token::NUM(0b000111000),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0010101000111000);

        let test_vec = vec![
            Token::INSTR(Op::LDI),
            Token::REGISTER(RegAddr::Five),
            Token::COMMA,
            Token::NUM(0b000111000),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1010101000111000);

        let test_vec = vec![
            Token::INSTR(Op::LDR),
            Token::REGISTER(RegAddr::Five),
            Token::COMMA,
            Token::REGISTER(RegAddr::Two),
            Token::COMMA,
            Token::NUM(0b111000),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0110101010111000);

        let test_vec = vec![
            Token::INSTR(Op::LEA),
            Token::REGISTER(RegAddr::Five),
            Token::COMMA,
            Token::NUM(0b000111000),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1110101000111000);
    }

    #[test]
    fn lex_store_instrs() {
        let test_vec = vec![
            Token::INSTR(Op::ST),
            Token::REGISTER(RegAddr::Five),
            Token::COMMA,
            Token::NUM(0b000111000),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0011101000111000);

        let test_vec = vec![
            Token::INSTR(Op::STI),
            Token::REGISTER(RegAddr::Five),
            Token::COMMA,
            Token::NUM(0b000111000),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1011101000111000);

        let test_vec = vec![
            Token::INSTR(Op::STR),
            Token::REGISTER(RegAddr::Five),
            Token::COMMA,
            Token::REGISTER(RegAddr::Two),
            Token::COMMA,
            Token::NUM(0b111000),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0111101010111000);
    }

    #[test]
    fn lex_not_instr() {
        let test_vec = vec![
            Token::INSTR(Op::NOT),
            Token::REGISTER(RegAddr::Five),
            Token::COMMA,
            Token::REGISTER(RegAddr::Zero),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1001101000111111);
    }

    #[test]
    fn lex_return_instrs() {
        let test_vec = vec![Token::INSTR(Op::RET), Token::SEMICOLON];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1100000111000000);

        let test_vec = vec![Token::INSTR(Op::RTI), Token::SEMICOLON];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1000000000000000);
    }

    #[test]
    fn lex_jump_instrs() {
        let test_vec = vec![
            Token::INSTR(Op::JMP),
            Token::REGISTER(RegAddr::Two),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b1100000010000000);

        let test_vec = vec![Token::INSTR(Op::JSR), Token::NUM(63), Token::SEMICOLON];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0100100000111111);

        let test_vec = vec![
            Token::INSTR(Op::JSRR),
            Token::REGISTER(RegAddr::Three),
            Token::SEMICOLON,
        ];
        let (label, instr) = lexer(test_vec);

        assert_eq!(label, None);
        assert_eq!(instr.unwrap().first().unwrap().value, 0b0100000011000000);
    }
}
