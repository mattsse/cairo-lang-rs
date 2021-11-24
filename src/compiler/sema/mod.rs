use crate::{
    compiler::sema::{ast::LangVisitor, identifiers::Identifiers},
    error::Result,
    parser::ast::{Builtin, Identifier},
    CairoFile,
};
use std::{borrow::Cow, fmt, path::PathBuf, rc::Rc};

pub mod ast;
pub mod identifiers;
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
    /// Manages identifiers for
    // TODO(mattssee): ideally this should be merged with the AST so that we have everything in one
    // place
    pub identifiers: Identifiers,
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
            identifiers: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CairoModule {
    pub module_name: Rc<ScopedName>,
    pub cairo_file: CairoFile,
}

impl CairoModule {
    pub fn new(module_name: ScopedName, cairo_file: CairoFile) -> Self {
        Self { module_name: Rc::new(module_name), cairo_file }
    }

    pub fn lang(&mut self) -> Result<Option<String>> {
        LangVisitor::lang(&mut self.cairo_file)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ScopedName(Identifier);

impl ScopedName {
    pub fn new(id: Identifier) -> Self {
        ScopedName(id)
    }

    pub fn main_scope() -> Self {
        Self::from_str("__main__")
    }

    pub fn size() -> Self {
        Self::from_str("SIZE")
    }

    /// The root scope, an empty scope
    pub fn root() -> Self {
        ScopedName(Default::default())
    }

    pub fn from_str(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();
        assert!(!s.is_empty(), "scope named requires >=1 identifier");
        ScopedName(s.split('.').map(str::to_string).collect())
    }

    pub fn last(&self) -> Option<&String> {
        self.0.last()
    }

    pub fn push(&mut self, name: String) {
        self.0.push(name)
    }

    pub fn name(&self) -> String {
        self.0.join(".")
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Removes the first name of the set and returns the remainder if there are any ids left
    pub fn split(mut self) -> (String, Option<ScopedName>) {
        let name = self.0.remove(0);
        if self.is_empty() {
            (name, None)
        } else {
            (name, Some(self))
        }
    }

    pub fn appended(mut self, id: impl Into<String>) -> Self {
        self.0.push(id.into());
        self
    }
}

impl fmt::Display for ScopedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(""))
    }
}
