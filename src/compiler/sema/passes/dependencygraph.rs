use crate::{
    compiler::{
        sema::{
            ast::macros::delegate_scope_tracking, identifiers::Identifiers, passes::Pass,
            PreprocessedProgram, ScopedName,
        },
        VResult, Visitable, Visitor,
    },
    error::Result,
    parser::ast::*,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct DependencyGraphPass {
    additional_scopes_to_compile: HashSet<ScopedName>,
}

impl DependencyGraphPass {
    pub fn new(additional_scopes_to_compile: HashSet<ScopedName>) -> Self {
        Self { additional_scopes_to_compile }
    }
}

impl Pass for DependencyGraphPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Dependency graph");
        for module in prg.modules.iter_mut() {
            prg.identifiers.scope_tracker_mut().enter_scope(module.module_name.clone());
            prg.identifiers.scope_tracker_mut().enter_lang(module.lang()?);

            let mut visitor = GraphVisitor::new(&mut prg.identifiers);
            module.cairo_file.visit(&mut visitor)?;

            prg.identifiers.scope_tracker_mut().exit_scope();
            prg.identifiers.scope_tracker_mut().exit_lang();
        }
        Ok(())
    }
}

/// Visitor that tracks the dependencies between scope and identifier
struct GraphVisitor<'a> {
    identifiers: &'a mut Identifiers,
    /// scope names to all the identifiers it uses.
    visited_identifiers: HashMap<ScopedName, Vec<ScopedName>>,
    /// the current function we're tracking
    current_function: Option<ScopedName>,
}

impl<'a> GraphVisitor<'a> {
    pub fn new(identifiers: &'a mut Identifiers) -> Self {
        Self {
            identifiers,
            visited_identifiers: Default::default(),
            current_function: Default::default(),
        }
    }

    fn add_identifier(&mut self) {
        todo!()
    }
}

impl<'a> Visitor for GraphVisitor<'a> {
    fn visit_import(&mut self, _import: &mut ImportDirective) -> VResult {
        todo!()
    }
    fn visit_function(&mut self, _: &mut FunctionDef) -> VResult {
        todo!()
    }

    fn visit_expr_dot(&mut self, _: &mut Expr, _: &mut String, _: Loc) -> VResult {
        todo!()
    }

    delegate_scope_tracking!();
}
