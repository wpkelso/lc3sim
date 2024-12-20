use crate::defs::LC3Word;

pub struct MaybeUnresolvedInstr {
    value: LC3Word,
    ///Label, Start offset, End offset
    bindings: Option<(String, u8, u8)>,
}

pub fn translate_line(line: &str) -> MaybeUnresolvedInstr {
    let mut trimmed_line = line.trim_end_matches(r";\N*").split_whitespace();

    MaybeUnresolvedInstr {
        value: 0x0,
        bindings: None,
    }
}

pub fn resolve_instr(instr: MaybeUnresolvedInstr) -> String {
    "00".to_string()
}
