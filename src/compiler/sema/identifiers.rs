use crate::compiler::sema::ScopedName;

/// Manages a list of identifiers
#[derive(Debug, Default)]
pub struct Identifiers {}

impl Identifiers {
    pub fn add_identifier(&mut self, name: ScopedName) {
        todo!()
    }
}

impl Identifiers {}

/// Represents a named identifier
#[derive(Debug, Clone)]
pub struct IdentifierDef {
    pub ty: IdentifierDefinitionType,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IdentifierDefinitionType {
    ConstDef,
    Label,
    Reference,
    LocalVar,
    TempVar,
    RValueRef,
}
