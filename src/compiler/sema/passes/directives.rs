use crate::{
    compiler::{
        sema::{passes::Pass, PreprocessedProgram},
        Visitable, Visitor,
    },
    error::Result,
};

#[derive(Debug, Default)]
pub struct DirectivesCollectorPass {}

impl DirectivesCollectorPass {}

impl Pass for DirectivesCollectorPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Directives Collector");
        for module in prg.modules.iter_mut() {
            module.cairo_file.visit(self)?;
        }
        Ok(())
    }
}

impl Visitor for DirectivesCollectorPass {}
