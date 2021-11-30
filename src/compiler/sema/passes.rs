use crate::compiler::sema::PreprocessedProgram;

use crate::{
    compiler::{
        sema::passes::{
            directives::DirectivesCollectorPass, identifier::IdentifierCollectorPass,
            import::ModuleCollectorPass, label::UniqueLabelPass,
            struct_collect::StructCollectorPass,
        },
        ModuleReader,
    },
    error::Result,
};
use ethers::core::k256::U256;
use std::fmt;

mod dependencygraph;
mod directives;
mod identifier;
mod import;
mod label;
mod preprocess;
mod struct_collect;

/// A manager for running passes
#[derive(Debug)]
pub struct PassManager {
    passes: Vec<Box<dyn Pass + 'static>>,
}

impl PassManager {
    pub fn starknet_pass_manager() -> Self {
        todo!()
    }

    pub fn run_on(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        for t in self.passes.iter_mut() {
            t.run(prg)?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct PassManagerBuilder {
    module_reader: Option<ModuleReader>,
}

impl PassManagerBuilder {
    /// Use a custom `ModuleReader`
    pub fn module_reader(mut self, module_reader: ModuleReader) -> Self {
        self.module_reader = Some(module_reader);
        self
    }

    pub fn build(self) -> PassManager {
        PassManager {
            passes: vec![
                Box::new(ModuleCollectorPass::new(self.module_reader.unwrap_or_default())),
                Box::new(UniqueLabelPass::default()),
                Box::new(IdentifierCollectorPass::default()),
                Box::new(DirectivesCollectorPass::default()),
                Box::new(StructCollectorPass::default()),
            ],
        }
    }
}

impl Default for PassManager {
    fn default() -> Self {
        PassManagerBuilder::default().build()
    }
}

impl From<U256> for PassManager {
    fn from(_prime: U256) -> Self {
        todo!()
    }
}

pub trait Pass: fmt::Debug {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()>;
}
