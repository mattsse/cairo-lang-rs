use crate::{
    compiler::sema::ScopedName,
    error::{CairoError, Result},
};
use std::{collections::HashMap, rc::Rc};

/// Manages a list of identifiers
#[derive(Debug, Default)]
pub struct Identifiers {
    root: Scope,
    identifiers: HashMap<ScopedName, Rc<IdentifierDefinitionType>>,
}

impl Identifiers {
    /// adds the given identifier def with the name to the current scope
    pub fn add_identifier(&mut self, name: ScopedName, ty: IdentifierDefinitionType) {
        let ty = Rc::new(ty);
        let dest = self.root.add_identifier(name, Rc::clone(&ty));
        self.identifiers.insert(dest, ty);
    }
}

/// A scope of identifiers
#[derive(Debug)]
pub struct Scope {
    /// name of the scope
    pub full_name: ScopedName,
    /// sub scopes inside this current scope
    pub subscopes: HashMap<String, Scope>,
    /// identifiers inside this scope
    pub identifiers: HashMap<String, Rc<IdentifierDefinitionType>>,
}

impl Scope {
    pub fn new(full_name: ScopedName) -> Self {
        Self { full_name, subscopes: Default::default(), identifiers: Default::default() }
    }

    /// Returns the direct child scope by name if it exists
    pub fn get_single_scope_mut(&mut self, name: &str) -> Option<&mut Scope> {
        self.subscopes.get_mut(name)
    }

    pub fn get_single_scope(&self, name: &str) -> Option<&Scope> {
        self.subscopes.get(name)
    }

    /// Attempts to find the corresponding scope with the given name
    ///
    /// Returns an error if
    pub fn get_scope(&self, name: &ScopedName) -> Result<&Scope> {
        if name.is_empty() {
            return Ok(self)
        }

        let (name, rem) = name.clone().split();
        if let Some(scope) = self.get_single_scope(&name) {
            if let Some(rem) = rem {
                return scope.get_scope(&rem)
            }
        }

        let full_name = self.full_name.clone().appended(name.clone());

        if !self.identifiers.contains_key(&name) {
            return Err(CairoError::MissingIdentifier(full_name))
        }

        Err(CairoError::NotScope(full_name))
    }

    fn add_subscope(&mut self, name: String) {
        let s = self.full_name.clone().appended(name.clone());
        self.subscopes.insert(name, Scope::new(s));
    }

    /// Adds an identifier to the set, the name is relative to the current scope
    pub fn add_identifier(
        &mut self,
        name: ScopedName,
        ty: Rc<IdentifierDefinitionType>,
    ) -> ScopedName {
        let (name, rem) = name.split();
        if let Some(rem) = rem {
            if self.get_single_scope_mut(&name).is_none() {
                self.add_subscope(name.clone());
            }
            self.get_single_scope_mut(&name).unwrap().add_identifier(rem, ty)
        } else {
            self.identifiers.insert(name.clone(), ty);
            self.full_name.clone().appended(name)
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Scope::new(ScopedName::root())
    }
}

/// Represents a named identifier
#[derive(Debug, Clone)]
pub struct IdentifierDef {
    pub ty: IdentifierDefinitionType,
    pub full_name: ScopedName,
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
