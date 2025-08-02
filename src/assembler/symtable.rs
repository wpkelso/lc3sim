use anyhow::Result;
use std::collections::HashMap;

use crate::defs::LC3MemAddr;

#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// The name of the object file this symtable is associated with.
    file: String,
    /// Set of key:value pairs that represents the symtable, indexable by Label.
    values: HashMap<String, LC3MemAddr>,
}

impl SymbolTable {
    fn export(&self) -> Result<()> {
        Ok(())
    }
}
