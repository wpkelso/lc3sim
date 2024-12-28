use crate::defs::LC3Word;

pub mod tokenizer;

pub struct MaybeUnresolvedInstr {
    value: LC3Word,
    ///Label, Start offset, End offset
    bindings: Option<(String, u8, u8)>,
}

pub fn translate_line(line: &str) -> MaybeUnresolvedInstr {
    todo!()
}

pub fn resolve_instr(instr: MaybeUnresolvedInstr) -> String {
    todo!()
}
