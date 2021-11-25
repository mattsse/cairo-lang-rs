use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    compiler::sema::{ast::StructDefinition, ScopedName},
    error::{CairoError, Result},
    parser::ast::{CairoType, Loc, StructDef},
};

/// Manages a list of identifiers
#[derive(Debug, Default)]
pub struct Identifiers {
    pub(crate) root: Scope,
    pub(crate) identifiers: HashMap<ScopedName, Rc<IdentifierDefinitionType>>,
}

impl Identifiers {
    /// adds the given identifier def with the name to the current scope
    pub fn add_identifier(&mut self, name: ScopedName, ty: IdentifierDefinitionType) {
        let ty = Rc::new(ty);
        let dest = self.root.add_identifier(name, Rc::clone(&ty));
        self.identifiers.insert(dest, ty);
    }

    /// Resolves a `CairoType` to a fully qualified name
    pub fn resolve_type(&mut self, cairo_type: CairoType) -> Result<CairoType> {
        todo!()
    }

    /// Adds a definition of an identifier that must be already registered
    pub fn add_name_definition(
        &mut self,
        name: ScopedName,
        ty: IdentifierDefinitionType,
        loc: Loc,
        require_registered_type: bool,
    ) -> Result<()> {
        if let Some(def) = self.get_by_full_name(&name) {
            if let Some(unresolved) = def.as_unresolved() {
                if !ty.has_matching_type(unresolved) {
                    return Err(CairoError::Preprocess(format!(
                        "Expected Identifier {} to be a {:?} but is {:?}",
                        name, unresolved, ty
                    )))
                }
            } else {
                return Err(CairoError::Redefinition(name.clone(), loc))
            }
        } else if require_registered_type {
            return Err(CairoError::Preprocess(format!("Identifier {} not found", name)))
        }

        // override the resolved type
        self.add_identifier(name, ty);
        Ok(())
    }

    /// Finds the identifier with the given name with aliases
    pub fn get(&self, name: &ScopedName) -> Result<ResolvedIdentifier> {
        let current_identifier = name.clone();
        let mut visited_identifiers = HashSet::from([current_identifier.clone()]);

        let mut resolved = self.root.get(&current_identifier)?;
        // resolve alias
        while resolved.ty.is_alias() {
            // check for cycles
            if visited_identifiers.contains(&current_identifier) {
                return Err(CairoError::Identifier(format!(
                    "Cyclic aliasing detected: {:?} {}",
                    visited_identifiers, current_identifier
                )))
            }
            visited_identifiers.insert(current_identifier.clone());
            resolved = self.root.get(&current_identifier)?;
        }
        Ok(resolved)
    }

    /// Attempts to find the scope with the given name, by resolving aliases
    pub fn get_scope(&self, name: &ScopedName) -> Result<&Scope> {
        let mut visited_identifiers = HashSet::<ScopedName>::default();
        let mut current_identifier = name.clone();
        loop {
            if visited_identifiers.contains(&current_identifier) {
                break
            }
            visited_identifiers.insert(current_identifier.clone());

            match self.root.get_scope(&current_identifier) {
                scope @ Ok(_) => return scope,
                Err(CairoError::NotScope(scope, rem, ty)) => {
                    if let IdentifierDefinitionType::Alias(destination) = ty {
                        if let Some(rem) = rem {
                            current_identifier = destination.appended(rem.to_string());
                        } else {
                            current_identifier = destination;
                        }
                    } else {
                        return Err(CairoError::NotScope(scope, rem, ty))
                    }
                }
                err => return err,
            }
        }
        Err(CairoError::Identifier(format!(
            "Alias resultion failed {:?}, {}",
            visited_identifiers, current_identifier
        )))
    }

    /// Returns the definition of an identifier
    ///
    /// NOTE: no aliasing at this point
    pub fn get_by_full_name(&self, name: &ScopedName) -> Option<Rc<IdentifierDefinitionType>> {
        if name.is_empty() {
            return None
        }
        let resolved = self.root.get(name).ok()?;
        if resolved.rem.is_some() {
            return None
        }
        Some(resolved.ty)
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

    /// Returns the identifier with the given name
    pub fn get(&self, name: &ScopedName) -> Result<ResolvedIdentifier> {
        let (name, rem) = name.clone().split();
        let canonical_name = self.full_name.clone().appended(name.clone());

        if let Some(ref rem) = rem {
            if let Some(scope) = self.get_single_scope(&name) {
                return scope.get(rem)
            }
        }

        if let Some(ty) = self.identifiers.get(&name).cloned() {
            return Ok(ResolvedIdentifier { ty, canonical_name, rem })
        }

        if self.subscopes.contains_key(&name) {
            return Err(CairoError::NotIdentifier(canonical_name))
        }

        Err(CairoError::MissingIdentifier(canonical_name))
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

        if let Some(ty) = self.identifiers.get(&name).cloned() {
            Err(CairoError::NotScope(full_name, rem, ty.as_ref().clone()))
        } else {
            Err(CairoError::MissingIdentifier(full_name))
        }
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IdentifierDefinitionType {
    ConstDef,
    Label,
    Reference,
    LocalVar,
    Function,
    Namespace,
    Struct(Option<StructDefinition>),
    TempVar,
    RValueRef,
    Alias(ScopedName),
    Unresolved(Box<IdentifierDefinitionType>),
}

impl IdentifierDefinitionType {
    pub fn is_const(&self) -> bool {
        matches!(self, IdentifierDefinitionType::ConstDef)
    }
    pub fn is_label(&self) -> bool {
        matches!(self, IdentifierDefinitionType::Label)
    }
    pub fn is_local_var(&self) -> bool {
        matches!(self, IdentifierDefinitionType::LocalVar)
    }
    pub fn is_temp_var(&self) -> bool {
        matches!(self, IdentifierDefinitionType::TempVar)
    }
    pub fn is_alias(&self) -> bool {
        matches!(self, IdentifierDefinitionType::Alias(_))
    }
    pub fn is_rvalue_ref(&self) -> bool {
        matches!(self, IdentifierDefinitionType::RValueRef)
    }
    pub fn is_function(&self) -> bool {
        matches!(self, IdentifierDefinitionType::Function)
    }
    pub fn is_namespace(&self) -> bool {
        matches!(self, IdentifierDefinitionType::Namespace)
    }
    pub fn is_struct(&self) -> bool {
        matches!(self, IdentifierDefinitionType::Struct(_))
    }
    pub fn is_reference(&self) -> bool {
        matches!(self, IdentifierDefinitionType::Reference)
    }
    pub fn is_unresolved(&self) -> bool {
        matches!(self, IdentifierDefinitionType::Unresolved(_))
    }

    pub fn as_unresolved(&self) -> Option<&IdentifierDefinitionType> {
        if let IdentifierDefinitionType::Unresolved(inner) = self {
            Some(&*inner)
        } else {
            None
        }
    }

    pub fn is_unresolved_reference(&self) -> bool {
        if let IdentifierDefinitionType::Unresolved(ty) = self {
            ty.is_reference()
        } else {
            false
        }
    }

    pub fn has_matching_type(&self, other: &IdentifierDefinitionType) -> bool {
        match self {
            IdentifierDefinitionType::ConstDef => other.is_const(),
            IdentifierDefinitionType::Label => other.is_label(),
            IdentifierDefinitionType::Reference => other.is_reference(),
            IdentifierDefinitionType::LocalVar => other.is_local_var(),
            IdentifierDefinitionType::Function => other.is_function(),
            IdentifierDefinitionType::Namespace => other.is_namespace(),
            IdentifierDefinitionType::Struct(_) => other.is_struct(),
            IdentifierDefinitionType::TempVar => other.is_temp_var(),
            IdentifierDefinitionType::RValueRef => other.is_rvalue_ref(),
            IdentifierDefinitionType::Alias(_) => other.is_alias(),
            IdentifierDefinitionType::Unresolved(_) => other.is_unresolved(),
        }
    }
}

#[derive(Debug)]
pub struct ResolvedIdentifier {
    pub ty: Rc<IdentifierDefinitionType>,
    pub canonical_name: ScopedName,
    pub rem: Option<ScopedName>,
}
