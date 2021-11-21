use crate::compiler::sema::ast::Visitor;
use crate::compiler::{ModuleReader, VResult, Visitable};
use crate::error::{CairoError, Result};
use crate::parser::ast::{Identifier, ImportDirective};
use crate::CairoFile;
use std::collections::HashMap;

/// A helper visitor type that can collect all imports of a given module
pub struct ImportCollector<'a> {
    reader: &'a ModuleReader,
    current_ancestors: Vec<String>,
    collected_files: HashMap<String, CairoFile>,
    langs: HashMap<String, Option<String>>,
}

impl<'a> ImportCollector<'a> {
    pub fn new(reader: &'a ModuleReader) -> Self {
        Self {
            reader,
            current_ancestors: Default::default(),
            collected_files: Default::default(),
            langs: Default::default(),
        }
    }

    /// Scans all imports of the given module recursively
    pub fn collect_imports(&mut self, current_module: impl Into<String>) -> Result<()> {
        let current_module = current_module.into();
        if self.current_ancestors.contains(&current_module) {
            return Err(CairoError::CircularDependencies(std::mem::take(
                &mut self.current_ancestors,
            )));
        }
        if self.collected_files.contains_key(&current_module) {
            // file already parsed
            return Ok(());
        }

        let (code, _) = self.reader.read(&current_module)?;
        let mut cairo_file = CairoFile::parse(&code)?;

        let lang = LangVisitor::lang(&mut cairo_file)?;

        // add current package to ancestors list before scanning its dependencies.
        self.current_ancestors.push(current_module.clone());

        // collect direct dependencies
        for pkg in DirectDependenciesCollector::deps(&mut cairo_file)? {
            self.collect_imports(&pkg)?;
            let same_directive = if let Some(l) = self.langs.get(&pkg) {
                l == &lang
            } else {
                true
            };
            if !same_directive {
                return Err(CairoError::InvalidImport(format!("importing modules with %lang directive {:?} must be from a module with the same directive", self.langs.get(&pkg))));
            }
        }

        self.current_ancestors.pop();
        self.collected_files
            .insert(current_module.clone(), cairo_file);
        self.langs.insert(current_module, lang);
        Ok(())
    }
}

/// A visitor that collects module names
#[derive(Default)]
struct DirectDependenciesCollector(Vec<String>);

impl DirectDependenciesCollector {
    fn deps(file: &mut CairoFile) -> Result<Vec<String>> {
        let mut v = Self::default();
        file.visit(&mut v)?;
        Ok(v.0)
    }
}

impl Visitor for DirectDependenciesCollector {
    fn visit_import(&mut self, import: &mut ImportDirective) -> VResult {
        self.0.push(import.name());
        Ok(())
    }
}

/// A visitor that returns the %lang directive of a cairo file
#[derive(Default)]
struct LangVisitor(Option<String>);

impl LangVisitor {
    fn lang(file: &mut CairoFile) -> Result<Option<String>> {
        let mut lang = Self::default();
        file.visit(&mut lang)?;
        Ok(lang.0)
    }
}

impl Visitor for LangVisitor {
    fn visit_lang(&mut self, id: &mut Identifier) -> VResult {
        let id = id.join(".");
        if self.0.is_some() {
            return Err(CairoError::msg(format!(
                "Found two %lang directives {}",
                id
            )));
        }
        self.0 = Some(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_reader() -> ModuleReader {
        let root = Path::new(&env!("CARGO_MANIFEST_DIR"));
        ModuleReader::new([root.join("common"), root.join("test-data/cairo-files")])
    }

    #[test]
    fn can_collect_imports() {
        let reader = test_reader();
        let mut imports = ImportCollector::new(&reader);
        imports.collect_imports("imports").unwrap();
        assert!(!imports.collected_files.is_empty());
    }
}
