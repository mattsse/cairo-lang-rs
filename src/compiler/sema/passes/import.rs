use crate::{
    compiler::{
        module_reader::CodeReader,
        sema::{
            ast::Visitor, passes::Pass, CairoContent, CairoModule, PreprocessedProgram, ScopedName,
        },
        ModuleReader, VResult, Visitable,
    },
    error::{CairoError, Result},
    parser::ast::{Identifier, ImportDirective},
    CairoFile,
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Debug, Default)]
pub struct ModuleCollectorPass {
    additional_modules: Vec<String>,
    reader: ModuleReader,
}

impl ModuleCollectorPass {
    pub fn new(reader: ModuleReader) -> Self {
        Self::with_modules(reader, Default::default())
    }

    pub fn with_modules(reader: ModuleReader, additional_modules: Vec<String>) -> Self {
        Self { reader, additional_modules }
    }
}

impl Pass for ModuleCollectorPass {
    fn run(&mut self, prg: &mut PreprocessedProgram) -> Result<()> {
        log::trace!("starting pass: ModuleCollector");
        let mut visited = HashSet::new();

        // resolve additional modules
        for module in &self.additional_modules {
            let mut collector = ImportCollector::new(&self.reader);
            collector.collect_imports(module)?;
            for (module_name, cairo_file) in collector.collected_files {
                if visited.insert(module_name.clone()) {
                    let scope = ScopedName::from_str(module_name);
                    prg.modules.push(CairoModule::new(scope, cairo_file));
                }
            }
        }

        // resolve source files
        for content in &prg.codes {
            let mut collector =
                ImportCollector::new(InputCodeReader { reader: &self.reader, content });
            let file_name = content.name();
            collector.collect_imports(file_name.clone())?;
            for (module_name, cairo_file) in collector.collected_files {
                let scope = if module_name == file_name {
                    prg.main_scope.clone()
                } else {
                    if !visited.insert(module_name.clone()) {
                        continue
                    }
                    ScopedName::from_str(module_name)
                };
                prg.modules.push(CairoModule::new(scope, cairo_file));
            }
        }

        Ok(())
    }
}

struct InputCodeReader<'a> {
    reader: &'a ModuleReader,
    content: &'a CairoContent,
}

impl<'a> CodeReader for InputCodeReader<'a> {
    fn read(&self, module: &str) -> Result<(String, PathBuf)> {
        if module == self.content.name() {
            Ok((self.content.code.clone(), self.content.path.clone()))
        } else {
            self.reader.read(module)
        }
    }
}

/// A helper visitor type that can collect all imports of a given module
struct ImportCollector<T> {
    reader: T,
    current_ancestors: Vec<String>,
    collected_files: HashMap<String, CairoFile>,
    langs: HashMap<String, Option<String>>,
}

impl<T: CodeReader> ImportCollector<T> {
    pub fn new(reader: T) -> Self {
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
            )))
        }
        if self.collected_files.contains_key(&current_module) {
            // file already parsed
            return Ok(())
        }

        let (code, _) = self.reader.read(&current_module)?;
        let mut cairo_file = CairoFile::parse(&code)?;

        let lang = LangVisitor::lang(&mut cairo_file)?;

        // add current package to ancestors list before scanning its dependencies.
        self.current_ancestors.push(current_module.clone());

        // collect direct dependencies
        for pkg in DirectDependenciesCollector::deps(&mut cairo_file)? {
            self.collect_imports(&pkg)?;
            let same_directive = if let Some(l) = self.langs.get(&pkg) { l == &lang } else { true };
            if !same_directive {
                return Err(CairoError::InvalidImport(format!("importing modules with %lang directive {:?} must be from a module with the same directive", self.langs.get(&pkg))));
            }
        }

        self.current_ancestors.pop();
        self.collected_files.insert(current_module.clone(), cairo_file);
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
            return Err(CairoError::msg(format!("Found two %lang directives {}", id)))
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
