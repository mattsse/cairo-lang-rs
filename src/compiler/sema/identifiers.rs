use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    compiler::{
        sema::{
            ast::{ScopeTracker, StructDefinition},
            ScopedName,
        },
        VResult, Visitor,
    },
    error::{CairoError, Result},
    parser::ast::{CairoType, FunctionDef, Loc, Namespace, PointerType, TypeStruct},
};

/// Manages a list of identifiers and types
#[derive(Debug, Default)]
pub struct Identifiers {
    /// keeps track of the scopes while traversing the AST
    pub(crate) scope_tracker: ScopeTracker,
    pub(crate) root: Scope,
    pub(crate) identifiers: HashMap<ScopedName, Rc<IdentifierDefinitionType>>,
}

impl Identifiers {
    pub fn resolved_identifiers(
        &self,
    ) -> impl Iterator<Item = (&ScopedName, &Rc<IdentifierDefinitionType>)> {
        self.identifiers.iter().filter(|(_, id)| !id.is_unresolved())
    }

    /// adds the given identifier def with the name to the current scope
    pub fn add_identifier(&mut self, name: ScopedName, ty: IdentifierDefinitionType) {
        let ty = Rc::new(ty);
        let dest = self.root.add_identifier(name, Rc::clone(&ty));
        self.identifiers.insert(dest, ty);
    }

    /// Resolves a `CairoType` to a fully qualified name
    pub fn resolve_type(&mut self, cairo_type: CairoType) -> Result<CairoType> {
        let ty = match cairo_type {
            CairoType::Felt => CairoType::Felt,
            CairoType::Id(ty) => {
                if ty.is_fully_resolved {
                    CairoType::Id(ty)
                } else {
                    let scope = ScopedName::new(ty.name);
                    let name = self.get_canonical_struct_name(&scope)?;
                    let ty = TypeStruct {
                        name: name.into_inner(),
                        is_fully_resolved: true,
                        loc: ty.loc,
                    };
                    CairoType::Id(ty)
                }
            }
            CairoType::Tuple(tuple) => CairoType::Tuple(
                tuple.into_iter().map(|ty| self.resolve_type(ty)).collect::<Result<_>>()?,
            ),
            CairoType::Pointer(ty) => {
                let is_single = ty.is_single();
                let ty = self.resolve_type(ty.into_pointee())?;
                let pointer =
                    if is_single { PointerType::Single(ty) } else { PointerType::Double(ty) };
                CairoType::Pointer(Box::new(pointer))
            }
        };
        Ok(ty)
    }

    /// Returns the canonical name for the struct given by scope in the current accessible_scopes
    pub fn get_canonical_struct_name(&self, struct_name: &ScopedName) -> Result<ScopedName> {
        let def = self.search(struct_name, self.scope_tracker.accessible_scopes())?;

        if def.ty.is_struct() || def.ty.is_unresolved_struct() {
            Ok(def.canonical_name)
        } else {
            Err(CairoError::Preprocess(format!(
                "Expected {} to a a struct, found {:?}",
                def.canonical_name, def.ty
            )))
        }
    }

    /// Returns the struct definition that corresponds to the given identifier.
    pub fn get_struct_definition(&self, struct_name: &ScopedName) -> Result<Rc<StructDefinition>> {
        let def = self.search(struct_name, self.scope_tracker.accessible_scopes())?;
        if !def.is_fully_parsed() {
            return Err(CairoError::Identifier(format!(
                "Unexpected remainder {:?} for {} of ty {:?}",
                def.rem, def.canonical_name, def.ty
            )))
        }
        if let Some(struct_def) = def.ty.as_struct() {
            Ok(struct_def)
        } else {
            Err(CairoError::Preprocess(format!(
                "Expected {} to be a struct definition but found {:?}",
                def.canonical_name, def.ty
            )))
        }
    }

    /// Returns the struct definition of a struct with no alias resultion
    fn get_struct_definition_no_alias(
        &self,
        struct_name: &ScopedName,
    ) -> Result<Rc<StructDefinition>> {
        let def = self
            .get_by_full_name(struct_name)
            .ok_or_else(|| CairoError::MissingIdentifier(struct_name.clone()))?;

        if let Some(struct_def) = def.as_struct() {
            Ok(struct_def)
        } else {
            Err(CairoError::Definition(
                struct_name.clone(),
                IdentifierDefinitionType::Struct(None),
                def.as_ref().clone(),
            ))
        }
    }

    pub fn get_struct_size(&self, struct_name: &ScopedName) -> Result<u64> {
        Ok(self.get_struct_definition(struct_name)?.size)
    }

    /// Returns the size of the given type
    pub fn get_size(&self, cairo_type: &CairoType) -> Result<u64> {
        match cairo_type {
            CairoType::Felt => Ok(CairoType::FELT_SIZE),
            CairoType::Id(type_struct) => {
                let scope = ScopedName::new(type_struct.name.clone());
                if type_struct.is_fully_resolved {
                    let def = self.get_struct_definition_no_alias(&scope)?;
                    Ok(def.size)
                } else {
                    self.get_struct_size(&scope)
                }
            }
            CairoType::Tuple(tuple) => {
                let mut size = 0;
                for ty in tuple {
                    size += self.get_size(ty)?;
                }
                Ok(size)
            }
            CairoType::Pointer(_) => Ok(CairoType::POINTER_SIZE),
        }
    }

    pub fn search_current_scopes(&self, name: &ScopedName) -> Result<ResolvedIdentifier> {
        self.search(name, self.scope_tracker.accessible_scopes())
    }

    /// Searches an identifier in the given accessible scopes
    pub fn search(
        &self,
        name: &ScopedName,
        accessible_scopes: &[Rc<ScopedName>],
    ) -> Result<ResolvedIdentifier> {
        for scope in accessible_scopes.iter().rev() {
            let id = scope.as_ref().clone().extended(name.clone());
            match self.get(&id) {
                res @ Ok(_) => return res,
                Err(CairoError::MissingIdentifier(err)) => {
                    // check whether if we're currently at the first item in the name of in the
                    // scope itself, in which case continue to the next accessible scope
                    let (name, _) = name.clone().rev_split();
                    if scope.as_ref().clone().extended(name).name().starts_with(&err.name()) {
                        continue
                    } else {
                        return Err(CairoError::MissingIdentifier(err))
                    }
                }
                res @ Err(_) => return res,
            }
        }
        let name = name.clone().rev_split().0;
        Err(CairoError::MissingIdentifier(name))
    }

    /// Searches a scope in the given accessible scopes
    pub fn search_scope(
        &self,
        name: &ScopedName,
        accessible_scopes: &[Rc<ScopedName>],
    ) -> Result<&Scope> {
        for scope in accessible_scopes.iter().rev() {
            let id = scope.as_ref().clone().extended(name.clone());
            match self.get_scope(&id) {
                res @ Ok(_) => return res,
                Err(CairoError::MissingIdentifier(err)) => {
                    // check whether if we're currently at the first item in the name of in the
                    // scope itself, in which case continue to the next accessible scope
                    let (name, _) = name.clone().rev_split();
                    if scope.as_ref().clone().extended(name).name().starts_with(&err.name()) {
                        continue
                    } else {
                        return Err(CairoError::MissingIdentifier(err))
                    }
                }
                res @ Err(_) => return res,
            }
        }
        let name = name.clone().rev_split().0;
        Err(CairoError::MissingIdentifier(name))
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
        let mut current_identifier = name.clone();
        let mut visited_identifiers = HashSet::from([current_identifier.clone()]);

        let mut resolved = self.root.get(&current_identifier)?;
        // resolve alias
        while let Some(alias) = resolved.ty.as_alias() {
            current_identifier = alias.clone();
            if let Some(rem) = resolved.rem {
                current_identifier = current_identifier.extended(rem);
            }
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
            "Alias resolution failed {:?}, {}",
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

    pub fn current_scope(&self) -> &Rc<ScopedName> {
        self.scope_tracker.current_scope()
    }

    pub fn scope_tracker_mut(&mut self) -> &mut ScopeTracker {
        &mut self.scope_tracker
    }
}

impl Visitor for Identifiers {
    fn enter_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.scope_tracker.enter_function(f)
    }

    fn exit_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.scope_tracker.exit_function(f)
    }

    fn enter_namespace(&mut self, n: &mut Namespace) -> VResult {
        self.scope_tracker.enter_namespace(n)
    }

    fn exit_namespace(&mut self, n: &mut Namespace) -> VResult {
        self.scope_tracker.exit_namespace(n)
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
    Struct(Option<Rc<StructDefinition>>),
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

    pub fn as_struct(&self) -> Option<Rc<StructDefinition>> {
        match self {
            IdentifierDefinitionType::Struct(s) => s.clone(),
            IdentifierDefinitionType::Unresolved(inner) => inner.as_struct(),
            _ => None,
        }
    }

    pub fn as_alias(&self) -> Option<&ScopedName> {
        match self {
            IdentifierDefinitionType::Alias(s) => Some(s),
            IdentifierDefinitionType::Unresolved(inner) => inner.as_alias(),
            _ => None,
        }
    }

    pub fn is_unresolved_reference(&self) -> bool {
        if let IdentifierDefinitionType::Unresolved(ty) = self {
            ty.is_reference()
        } else {
            false
        }
    }

    pub fn is_unresolved_struct(&self) -> bool {
        if let IdentifierDefinitionType::Unresolved(ty) = self {
            ty.is_struct()
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

impl ResolvedIdentifier {
    pub fn is_fully_parsed(&self) -> bool {
        self.rem.is_none()
    }
}
