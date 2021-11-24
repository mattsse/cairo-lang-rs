use crate::{
    compiler::{
        sema::{
            ast::ScopeTracker, identifiers::IdentifierDefinitionType, passes::Pass, Identifiers,
            PreprocessedProgram, ScopedName,
        },
        VResult, Visitable, Visitor,
    },
    error::{CairoError, Result},
    parser::ast::*,
};

/// Resolves identifiers for cairo code elements.
#[derive(Debug)]
pub struct IdentifierCollectorPass {}

impl IdentifierCollectorPass {}

impl Pass for IdentifierCollectorPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Identifier Collector");
        let mut scope_tracker = ScopeTracker::default();
        for module in prg.modules.iter_mut() {
            scope_tracker.enter_scope(module.module_name.clone());
            scope_tracker.enter_lang(module.lang()?);

            let mut visitor =
                IdVisitor { identifiers: &mut prg.identifiers, scope_tracker: &mut scope_tracker };

            // TODO store the full name directly in the AST use a hashmap to track potential
            // duplicates  need to add pub fullname: Option<ScopedNamed> to various AST
            // type, and also store in prg identifiers

            module.cairo_file.visit(&mut visitor)?;

            scope_tracker.exit_scope();
            scope_tracker.exit_lang();
        }
        Ok(())
    }
}

/// A scope aware AST visitor that resolves full names of identifiers
struct IdVisitor<'a> {
    identifiers: &'a mut Identifiers,
    /// keeps track of the current scope
    scope_tracker: &'a mut ScopeTracker,
}

impl<'a> IdVisitor<'a> {
    fn current_identifier(&self, identifier: String) -> ScopedName {
        self.scope_tracker.next_scope(identifier)
    }

    /// adds an identifier to the underlying
    fn add_identifier(
        &mut self,
        name: ScopedName,
        ty: IdentifierDefinitionType,
        loc: Loc,
    ) -> VResult {
        if let Some(existing_def) = self.identifiers.get_by_full_name(&name) {
            if !existing_def.is_unresolved() || !ty.is_unresolved() {
                return Err(CairoError::Preprocess(format!("Redefinition of {} at {:?}", name, loc)))
            }
            if !existing_def.is_reference() || !ty.is_reference() {
                return Err(CairoError::Preprocess(format!("Redefinition of {} at {:?}", name, loc)))
            }
        }
        self.identifiers.add_identifier(name, ty);
        Ok(())
    }

    fn add_unresolved_identifier(
        &mut self,
        name: ScopedName,
        ty: IdentifierDefinitionType,
        loc: Loc,
    ) -> VResult {
        self.add_identifier(name, IdentifierDefinitionType::Unresolved(Box::new(ty)), loc)
    }
}

impl<'a> Visitor for IdVisitor<'a> {
    fn visit_const_def(&mut self, _: &mut ConstantDef) -> VResult {
        Ok(())
    }

    fn visit_struct_def(&mut self, _: &mut Struct) -> VResult {
        Ok(())
    }

    fn visit_with(&mut self, el: &mut WithStatement) -> VResult {
        for id in &el.ids {
            if let Some(alias) = id.alias.clone() {
                self.add_unresolved_identifier(
                    self.current_identifier(alias),
                    IdentifierDefinitionType::Reference,
                    el.loc,
                )?;
            }
        }
        Ok(())
    }

    fn visit_label(&mut self, id: &mut Identifier, loc: Loc) -> VResult {
        // self.add_identifier(
        //     self.current_identifier(item.id.clone()),
        //     IdentifierDefinitionType::Alias(alias_dest),
        //     el.loc,
        // )

        Ok(())
    }

    fn visit_let(&mut self, _: &mut RefBinding, _: &mut RValue) -> VResult {
        Ok(())
    }

    fn visit_import(&mut self, el: &mut ImportDirective) -> VResult {
        for item in el.aliased_identifier() {
            let alias_dest = ScopedName::new(el.path.clone()).appended(item.id.clone());

            // ensure destination is a valid id
            if self.identifiers.get_by_full_name(&alias_dest).is_none() {
                let _ = self.identifiers.get_scope(&alias_dest)?;
            }

            self.add_identifier(
                self.current_identifier(item.id.clone()),
                IdentifierDefinitionType::Alias(alias_dest),
                el.loc,
            )?;
        }
        Ok(())
    }

    fn enter_function(&mut self, f: &mut FunctionDef) -> VResult {
        self.scope_tracker.enter_function(f)
    }

    fn visit_function(&mut self, _import: &mut FunctionDef) -> VResult {
        Ok(())
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

    fn visit_if(&mut self, el: &mut IfStatement) -> VResult {
        let label_neq = el.label_neq.clone().ok_or(CairoError::MissingLabel(el.loc))?;
        let label_end = el.label_end.clone().ok_or(CairoError::MissingLabel(el.loc))?;
        self.add_unresolved_identifier(
            self.current_identifier(label_neq),
            IdentifierDefinitionType::Label,
            el.loc,
        )?;
        self.add_unresolved_identifier(
            self.current_identifier(label_end),
            IdentifierDefinitionType::Label,
            el.loc,
        )
    }

    fn visit_local_var(&mut self, _: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        Ok(())
    }

    fn visit_temp_var(&mut self, _: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        Ok(())
    }
}
