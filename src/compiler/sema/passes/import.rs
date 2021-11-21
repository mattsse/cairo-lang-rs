use crate::compiler::sema::ast::Visitor;
use crate::compiler::ModuleReader;

/// A helper visitor type that can collect all imports of a given module
pub struct ImportCollector<'a> {
    reader: &'a ModuleReader,
    current_ancestors: Vec<String>,
}

impl<'a> ImportCollector<'a> {
    pub fn new(reader: &'a ModuleReader) -> Self {
        Self {
            reader,
            current_ancestors: vec![],
        }
    }

    /// Scans all imports of the
    pub fn collect_imports(&mut self, module: &str) -> eyre::Result<()> {
        todo!()
    }
}

impl<'a> Visitor for ImportCollector<'a> {}
