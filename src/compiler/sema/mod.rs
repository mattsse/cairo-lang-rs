use crate::compiler::module_reader::ModuleReader;
use crate::parser::ast::{Builtin, Identifier};
use crate::CairoFile;
use std::io::BufRead;
use std::path::{Path, PathBuf};

mod ast;
pub mod passes;

/// When assembling a cairo file, this holds all the resolved info.
#[derive(Debug)]
pub struct PreprocessedProgram {
    pub main_scope: ScopedName,
    pub modules: Vec<CairoModule>,
    /// various cairo builtins
    pub builtins: Option<Vec<Builtin>>,
}

impl PreprocessedProgram {
    /// Preprocesses a list of cairo files
    pub fn preprocess<I>(mut self, codes: I, module_reader: &mut ModuleReader) -> eyre::Result<Self>
    where
        I: IntoIterator<Item = (String, PathBuf)>,
    {
        // TODO get rid of this step in favor of a compiler pass?
        // initialize the preprocessed program by parsing all files into modules
        todo!()
    }

    pub fn new(main_scope: ScopedName) -> Self {
        Self {
            main_scope,
            modules: vec![],
            builtins: None,
        }
    }
}

impl Default for PreprocessedProgram {
    fn default() -> Self {
        Self::new(ScopedName::main_scope())
    }
}

#[derive(Debug, Clone)]
pub struct CairoModule {
    pub cairo_file: CairoFile,
    pub module_name: ScopedName,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScopedName(pub Identifier);

impl ScopedName {
    pub fn main_scope() -> Self {
        Self::from_str("__main__")
    }

    pub fn size() -> Self {
        Self::from_str("SIZE")
    }

    pub fn from_str(s: impl AsRef<str>) -> Self {
        ScopedName(s.as_ref().split('.').map(str::to_string).collect())
    }

    pub fn last(&self) -> Option<&String> {
        self.0.last()
    }
}
