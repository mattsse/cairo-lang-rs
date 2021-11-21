use crate::{
    parser::ast::{Builtin, Identifier},
    CairoFile,
};
use std::{borrow::Cow, path::PathBuf};

pub mod ast;
pub mod passes;

#[derive(Debug, Clone)]
pub struct CairoContent {
    /// code content of the file
    pub code: String,
    /// location the code was read from
    pub path: PathBuf,
}

impl CairoContent {
    pub fn new(code: String, path: PathBuf) -> Self {
        debug_assert!(path.file_stem().is_some(), "File must have a name");
        Self { code, path }
    }

    /// Returns the file stem of the source file
    pub fn name(&self) -> Cow<'_, str> {
        self.path.file_stem().unwrap().to_string_lossy()
    }
}

/// When assembling a cairo file, this holds all the resolved info.
#[derive(Debug)]
pub struct PreprocessedProgram {
    /// input code content
    pub codes: Vec<CairoContent>,
    pub main_scope: ScopedName,
    pub modules: Vec<CairoModule>,
    /// various cairo builtins
    pub builtins: Option<Vec<Builtin>>,
}

impl PreprocessedProgram {
    /// Preprocesses a list of cairo files
    pub fn new<I>(main_scope: ScopedName, codes: I) -> Self
    where
        I: IntoIterator<Item = (String, PathBuf)>,
    {
        Self {
            codes: codes.into_iter().map(|(c, p)| CairoContent::new(c, p)).collect(),
            main_scope,
            modules: Default::default(),
            builtins: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CairoModule {
    pub module_name: ScopedName,
    pub cairo_file: CairoFile,
}

impl CairoModule {
    pub fn new(module_name: ScopedName, cairo_file: CairoFile) -> Self {
        Self { module_name, cairo_file }
    }
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
