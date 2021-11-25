use crate::{
    compiler::{
        sema::{passes::Pass, PreprocessedProgram},
        VResult, Visitable, Visitor,
    },
    error::{CairoError, Result},
    parser::ast::{Builtin, Loc}
};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct DirectivesCollectorPass {
    builtins: Vec<Builtin>,
    builtins_set: bool,
}

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

impl Visitor for DirectivesCollectorPass {
    fn visit_builtins(&mut self, builtins: &mut [Builtin], loc: Loc) -> VResult {
        if self.builtins_set {
            return Err(CairoError::Preprocess(format!(
                "Redefinition of builtins directive: {}",
                loc
            )))
        }

        let mut unique_builtins = HashSet::new();
        for builtin in builtins.iter() {
            if !unique_builtins.insert(builtin) {
                return Err(CairoError::Preprocess(format!(
                    "Builtin {} appears twice in builtins directive",
                    builtin
                )))
            }
        }
        self.builtins = builtins.to_vec();

        Ok(())
    }
}
