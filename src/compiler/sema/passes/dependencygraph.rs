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

struct GraphVisitor<'a> {
    identifiers: &'a mut Identifiers,
    visited_identifiers: HashMap<ScopedName, Vec<ScopedName>>,
}

impl<'a> GraphVisitor<'a> {
    pub fn new(identifiers: &'a mut Identifiers) -> Self {
        Self { identifiers, visited_identifiers: Default::default() }
    }
}

impl<'a> Visitor for GraphVisitor<'a> {
    delegate_scope_tracking!();
}
