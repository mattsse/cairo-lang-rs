use crate::{
    compiler::{
        sema::{passes::Pass, PreprocessedProgram},
        Visitable, Visitor,
    },
    error::Result,
};
use std::collections::HashSet;

#[derive(Debug)]
pub struct PreprocessPass {
    /// A set of decorators that may appear before a function declaration
    pub supported_decorators: HashSet<String>,
}

impl PreprocessPass {}

impl Pass for PreprocessPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Preprocess");
        for module in prg.modules.iter_mut() {
            module.cairo_file.visit(self)?;
        }
        Ok(())
    }
}

impl Visitor for PreprocessPass {}
