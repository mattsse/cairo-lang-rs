use crate::{
    compiler::{
        sema::{passes::Pass, PreprocessedProgram},
        VResult, Visitable, Visitor,
    },
    error::Result,
    parser::ast::IfStatement,
};

/// Adds unique labels to `IfStatements`.
#[derive(Debug, Default)]
pub struct UniqueLabelPass {
    label_ctn: u64,
}

impl UniqueLabelPass {
    fn next_label(&mut self) -> String {
        let label = format!("_anon_label{}", self.label_ctn);
        self.label_ctn += 1;
        label
    }
}

impl Pass for UniqueLabelPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: ModuleCollector");
        for module in prg.modules.iter_mut() {
            module.cairo_file.visit(self)?;
        }
        Ok(())
    }
}

impl Visitor for UniqueLabelPass {
    fn visit_if(&mut self, stmt: &mut IfStatement) -> VResult {
        stmt.label_neq = Some(self.next_label());
        stmt.label_end = Some(self.next_label());
        Ok(())
    }
}
