use crate::{
    compiler::{
        sema::{passes::Pass, PreprocessedProgram},
        Visitable, Visitor,
    },
    error::Result,
};
use std::collections::HashSet;

#[derive(Debug)]
pub struct DependencyGraphPass;

impl DependencyGraphPass {}

impl Pass for DependencyGraphPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Dependency graph");
        for module in prg.modules.iter_mut() {
            module.cairo_file.visit(self)?;
        }
        Ok(())
    }
}

impl Visitor for DependencyGraphPass {}
