use crate::{
    compiler::{
        sema::{passes::Pass, PreprocessedProgram}, Visitable, Visitor,
    },
    error::Result,
};

/// Manages identifiers for cairo code elements.
#[derive(Debug)]
pub struct IdentifierCollectorPass {}

impl IdentifierCollectorPass {}

impl Pass for IdentifierCollectorPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Identifier Collector");
        for module in prg.modules.iter_mut() {
            module.cairo_file.visit(self)?;
        }
        Ok(())
    }
}

impl Visitor for IdentifierCollectorPass {
    // TODO get the identifiers from the code element
}
