use crate::{
    compiler::{
        sema::{ast::ScopeTracker, passes::Pass, Identifiers, PreprocessedProgram, ScopedName},
        VResult, Visitable, Visitor,
    },
    error::Result,
    parser::ast::*,
};
use std::rc::Rc;

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
    scope_tracker: &'a mut ScopeTracker,
}

impl<'a> IdVisitor<'a> {
    fn get_identifier(&self, identifier: String) -> ScopedName {
        self.scope_tracker.next_scope(identifier)
    }
}

impl<'a> Visitor for IdVisitor<'a> {
    fn visit_const_def(&mut self, _: &mut ConstantDef) -> VResult {
        Ok(())
    }

    fn visit_label(&mut self, _: &mut Identifier) -> VResult {
        Ok(())
    }

    fn visit_let(&mut self, _: &mut RefBinding, _: &mut RValue) -> VResult {
        Ok(())
    }

    fn visit_local_var(&mut self, _: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        Ok(())
    }

    fn visit_temp_var(&mut self, _: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        Ok(())
    }

    fn visit_function(&mut self, _import: &mut FunctionDef) -> VResult {
        Ok(())
    }

    fn visit_struct_def(&mut self, _: &mut Struct) -> VResult {
        Ok(())
    }

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
