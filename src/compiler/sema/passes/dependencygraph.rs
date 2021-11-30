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
use std::collections::HashSet;

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
        // for module in prg.modules.iter_mut() {
        //     module.cairo_file.visit(self)?;
        // }
        Ok(())
    }
}

struct GraphVisitor<'a> {
    identifiers: &'a mut Identifiers,
}

impl<'a> Visitor for GraphVisitor<'a> {
    delegate_scope_tracking!();
}
