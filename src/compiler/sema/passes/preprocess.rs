use crate::{
    compiler::{
        sema::{
            ast::macros::delegate_scope_tracking, identifiers::Identifiers, passes::Pass,
            PreprocessedProgram,
        },
        Visitable, Visitor,
    },
    error::Result,
};
use ethers::types::U256;
use std::collections::HashSet;

/// Collects extra information during preprocessing.
#[derive(Debug)]
pub struct AuxiliaryInfo {}

/// The pass that does the actual preprocessing
#[derive(Debug)]
pub struct PreprocessPass {
    /// The prime to compile for
    pub prime: U256,
    /// A set of decorators that may appear before a function declaration
    pub supported_decorators: HashSet<String>,
}

impl PreprocessPass {}

impl Pass for PreprocessPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: Preprocessor");
        for module in prg.modules.iter_mut() {
            prg.identifiers.scope_tracker_mut().enter_scope(module.module_name.clone());
            prg.identifiers.scope_tracker_mut().enter_lang(module.lang()?);

            let mut visitor = PreprocessVisitor { identifiers: &mut prg.identifiers };
            module.cairo_file.visit(&mut visitor)?;

            prg.identifiers.scope_tracker_mut().exit_scope();
            prg.identifiers.scope_tracker_mut().exit_lang();
        }
        Ok(())
    }
}

struct PreprocessVisitor<'a> {
    identifiers: &'a mut Identifiers,
}

impl<'a> Visitor for PreprocessVisitor<'a> {
    delegate_scope_tracking!();
}
