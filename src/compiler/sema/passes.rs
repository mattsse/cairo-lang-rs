use crate::compiler::sema::PreprocessedProgram;

use crate::error::Result;
use ethers::core::k256::U256;
use std::fmt;

mod import;
mod label;

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

pub trait Pass: fmt::Debug {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()>;
}
