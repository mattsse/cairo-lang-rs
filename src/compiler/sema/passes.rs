use crate::compiler::sema::ast::Visitor;
use crate::compiler::sema::PreprocessedProgram;
use crate::compiler::ModuleReader;
use ethers::core::k256::U256;
use std::fmt::Debug;

mod import;

/// A manager for running passes
#[derive(Debug)]
pub struct PassManager {
    passes: Vec<Box<dyn Pass + 'static>>,
}

impl PassManager {
    pub fn starknet_pass_manager() -> Self {
        todo!()
    }

    pub fn run_on(&mut self, prg: &mut PreprocessedProgram) {
        for t in self.passes.iter_mut() {
            t.run(prg);
        }
    }
}

impl Default for PassManager {
    fn default() -> Self {
        todo!()
    }
}

impl From<U256> for PassManager {
    fn from(_prime: U256) -> Self {
        todo!()
    }
}

pub trait Pass: Debug {
    fn run(&mut self, prg: &mut PreprocessedProgram);
}

#[derive(Debug)]
pub struct ModuleCollector {
    additional_modules: Vec<String>,
    reader: ModuleReader,
}
