use crate::{
    compiler::{
        sema::{passes::Pass, PreprocessedProgram},
        VResult, Visitable, Visitor,
    },
    error::Result,
    parser::ast::{
        ConstantDef, Expr, Identifier, RValue,
        RefBinding, TypedIdentifier,
    },
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
    fn visit_const_def(&mut self, _: &mut ConstantDef) -> VResult {
        Ok(())
    }

    fn visit_label(&mut self, _: &mut Identifier) -> VResult {
        Ok(())
    }

    fn visit_let(&mut self, _: &mut RefBinding, _: &mut RValue) -> VResult {
        todo!()
    }

    fn visit_local_var(&mut self, _: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        Ok(())
    }

    fn visit_temp_var(&mut self, _: &mut TypedIdentifier, _: &mut Option<Expr>) -> VResult {
        Ok(())
    }
}
