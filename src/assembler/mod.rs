use crate::{
    defs::{LC3Word, Op, PseudoOp, RegAddr},
    instruction::{
        ADD_OPCODE, ALL_JUMP_OPCODES, ALL_LOAD_OPCODES, ALL_STORE_OPCODES, AND_OPCODE, JSR_OPCODE,
        NOT_OPCODE,
    },
};
use anyhow::{bail, Result};
use strum_macros::EnumDiscriminants;

pub mod lexer;
pub mod tokenizer;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Binding {
    pub label: String,
    pub begin_offset: u8,
    pub end_offset: u8,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MaybeUnresolvedInstr {
    value: LC3Word,
    ///Label, Start offset, End offset
    bindings: Vec<(String, u8, u8)>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ExpectItem {
    Code(u8),
    Reg(u8),
    Offset(u8, u8),
    RegOrOffset(u8, u8),
    Comma,
    Semicolon,
    Bits(LC3Word),
}

impl ExpectItem {
    fn test(&self, token: Token) -> Result<TokenCheckResult> {
        match self {
            ExpectItem::Code(val) => Ok(TokenCheckResult::Value((*val as u16) << 12)),
            ExpectItem::Reg(shift) => token.is_register(*shift),
            ExpectItem::Comma => {
                if token.is_comma() {
                    Ok(TokenCheckResult::Value(0b0))
                } else {
                    bail!("Expected a comma, but it wasn't found!")
                }
            }
            ExpectItem::Semicolon => {
                if token.is_semicolon() {
                    Ok(TokenCheckResult::Value(0b0))
                } else {
                    bail!("Expect a semicolon, but it wasn't found!")
                }
            }
            ExpectItem::RegOrOffset(shift, max_len) => {
                token.is_register_or_offset(*shift, *max_len)
            }
            ExpectItem::Offset(shift, max_len) => token.is_offset(*shift, *max_len),
            ExpectItem::Bits(bits) => Ok(TokenCheckResult::Value(*bits)),
        }
    }
}

impl Op {
    fn get_sequence(&self) -> Vec<ExpectItem> {
        let add_sequence = vec![
            ExpectItem::Code(ADD_OPCODE),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Reg(6),
            ExpectItem::Comma,
            ExpectItem::RegOrOffset(0, 5),
            ExpectItem::Semicolon,
        ];

        let and_sequence = vec![
            ExpectItem::Code(AND_OPCODE),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Reg(6),
            ExpectItem::Comma,
            ExpectItem::RegOrOffset(0, 5),
            ExpectItem::Semicolon,
        ];

        let ld_sequence = vec![
            ExpectItem::Code(ALL_LOAD_OPCODES[0]),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Offset(0, 9),
            ExpectItem::Semicolon,
        ];

        let ldi_sequence = vec![
            ExpectItem::Code(ALL_LOAD_OPCODES[1]),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Offset(0, 9),
            ExpectItem::Semicolon,
        ];

        let ldr_sequence = vec![
            ExpectItem::Code(ALL_LOAD_OPCODES[2]),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Reg(6),
            ExpectItem::Comma,
            ExpectItem::Offset(0, 6),
            ExpectItem::Semicolon,
        ];

        let lea_sequence = vec![
            ExpectItem::Code(ALL_LOAD_OPCODES[3]),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Offset(0, 9),
            ExpectItem::Semicolon,
        ];

        let st_sequence = vec![
            ExpectItem::Code(ALL_STORE_OPCODES[0]),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Offset(0, 9),
            ExpectItem::Semicolon,
        ];

        let sti_sequence = vec![
            ExpectItem::Code(ALL_STORE_OPCODES[1]),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Offset(0, 9),
            ExpectItem::Semicolon,
        ];

        let str_sequence = vec![
            ExpectItem::Code(ALL_STORE_OPCODES[2]),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Reg(6),
            ExpectItem::Comma,
            ExpectItem::Offset(0, 6),
            ExpectItem::Semicolon,
        ];

        let not_sequence = vec![
            ExpectItem::Code(NOT_OPCODE),
            ExpectItem::Reg(9),
            ExpectItem::Comma,
            ExpectItem::Reg(6),
            ExpectItem::Bits(0b111111),
            ExpectItem::Semicolon,
        ];

        let jmp_sequence = vec![
            ExpectItem::Code(ALL_JUMP_OPCODES[0]),
            ExpectItem::Reg(6),
            ExpectItem::Semicolon,
        ];

        let jsr_sequence = vec![
            ExpectItem::Code(JSR_OPCODE),
            ExpectItem::Bits(0b1 << 11),
            ExpectItem::Offset(0, 11),
            ExpectItem::Semicolon,
        ];

        let jsrr_sequence = vec![ExpectItem::Code(JSR_OPCODE), ExpectItem::Reg(6)];

        let ret_sequence = vec![
            ExpectItem::Code(ALL_JUMP_OPCODES[2]),
            ExpectItem::Bits(0b111 << 6),
            ExpectItem::Semicolon,
        ];

        let rti_sequence = vec![ExpectItem::Code(ALL_JUMP_OPCODES[1]), ExpectItem::Semicolon];

        match self {
            Op::ADD => add_sequence,
            Op::AND => and_sequence,
            Op::LD => ld_sequence,
            Op::LDI => ldi_sequence,
            Op::LDR => ldr_sequence,
            Op::LEA => lea_sequence,
            Op::ST => st_sequence,
            Op::STI => sti_sequence,
            Op::STR => str_sequence,
            Op::NOT => not_sequence,
            Op::JMP => jmp_sequence,
            Op::JSR => jsr_sequence,
            Op::JSRR => jsrr_sequence,
            Op::RET => ret_sequence,
            Op::RTI => rti_sequence,
            _ => todo!(),
        }
    }
}

impl MaybeUnresolvedInstr {
    fn new_from_chain(mut chain: Vec<Token>) -> MaybeUnresolvedInstr {
        if let Token::INSTR(op) = chain[0] {
            let sequence = op.get_sequence();

            // Because we include the insertion of specified bits in the sequence,
            // we need to add some Token::None into the iterator to properly test the chain
            for (i, element) in sequence.iter().enumerate() {
                if let ExpectItem::Bits(_) = element {
                    println!("Inserting Token::NONE at {}", i);
                    chain.insert(i, Token::NONE);
                }
            }

            let mut results: Vec<TokenCheckResult> = Vec::new();
            let test_chain = sequence.iter().zip(chain.iter());
            for (expected, token) in test_chain {
                println!("Expecting {:?}, Found {:?}", expected, token);

                results.push(expected.test(token.clone()).unwrap());
            }

            let mut values: Vec<LC3Word> = Vec::new();
            let mut bindings: Vec<Binding> = Vec::new();
            for result in results {
                match result {
                    TokenCheckResult::Value(val) => values.push(val),
                    TokenCheckResult::Binding(binding) => bindings.push(binding),
                }
            }

            let mut instr = MaybeUnresolvedInstr {
                value: 0b0,
                bindings: Vec::new(),
            };

            instr.flatten_values(values);

            return instr;
        } else {
            todo!()
        }
    }

    /// Flattens a given Vec of LC3Words into &self
    fn flatten_values(&mut self, values: Vec<LC3Word>) {
        for value in values {
            self.value |= value;
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, EnumDiscriminants)]
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
    NONE,
}

enum TokenCheckResult {
    Value(LC3Word),
    Binding(Binding),
}

impl Token {
    fn is_string(&self) -> bool {
        if let Token::STRING(_) = self {
            true
        } else {
            false
        }
    }

    fn is_comma(&self) -> bool {
        if let Token::COMMA = self {
            true
        } else {
            false
        }
    }

    fn is_semicolon(&self) -> bool {
        if let Token::SEMICOLON = self {
            true
        } else {
            false
        }
    }

    fn is_register(&self, shift: u8) -> Result<TokenCheckResult> {
        if let Token::REGISTER(reg) = self {
            let mut value = 0b0;
            value |= LC3Word::from(*reg) << shift;
            Ok(TokenCheckResult::Value(value))
        } else {
            bail!("NOT A REGISTER")
        }
    }

    fn is_offset(&self, shift: u8, max_len: u8) -> Result<TokenCheckResult> {
        let result: TokenCheckResult;

        if let Token::NUM(num) = self {
            let mut value = 0b0;
            let max_mask = 1 << (max_len + 1);

            if *num < max_mask {
                value |= num << shift;
                result = TokenCheckResult::Value(value);
            } else {
                bail!("TOO BIG")
            }
        } else if let Token::STRING(label) = self {
            let binding = Binding {
                label: label.clone(),
                begin_offset: shift + max_len,
                end_offset: shift,
            };
            result = TokenCheckResult::Binding(binding);
        } else {
            bail!("NOT OFFSET")
        }

        Ok(result)
    }

    fn is_register_or_offset(&self, shift: u8, max_len: u8) -> Result<TokenCheckResult> {
        let result: TokenCheckResult;

        if let Token::REGISTER(reg) = self {
            let mut value = 0b0;
            value |= LC3Word::from(*reg) << shift;
            result = TokenCheckResult::Value(value);
        } else if let Token::NUM(num) = self {
            let mut value = 0b0;
            let max_mask = 1 << (max_len + 1);

            if *num < max_mask {
                value |= num << shift;
                value |= 1 << max_len;
                result = TokenCheckResult::Value(value);
            } else {
                bail!("TOO BIG")
            }
        } else if let Token::STRING(label) = self {
            let binding = Binding {
                label: label.clone(),
                begin_offset: shift + max_len,
                end_offset: shift,
            };
            result = TokenCheckResult::Binding(binding);
        } else {
            bail!("NOT REGISTER OR OFFSET")
        }

        Ok(result)
    }
}

pub fn translate_line(line: &str) -> MaybeUnresolvedInstr {
    todo!()
}

pub fn resolve_instr(instr: MaybeUnresolvedInstr) -> String {
    todo!()
}
