use crate::{
    compiler::{
        constants::{CAIRO_FILE_EXTENSION, LIBS_DIR_ENVVAR},
        sema::ScopedName,
    },
    error::{CairoError, Result},
};
use std::{collections::HashMap, fs, path::PathBuf};

/// Helper types that's used to read module files based their names
///
/// In oder to be able to properly resolve the modules, they must be stored under one of the allowed
/// paths.
#[derive(Debug, Clone)]
pub struct ModuleReader {
    /// where to look for paths
    paths: Vec<PathBuf>,
    resolved_modules: HashMap<String, ()>,
}

impl ModuleReader {
    pub fn new<I, P>(paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        Self {
            paths: paths.into_iter().map(Into::into).collect(),
            resolved_modules: Default::default(),
        }
    }

    /// Attempts to find the corresponding file for the given module
    pub fn find(&self, module: impl AsRef<str>) -> Option<PathBuf> {
        let scope = ScopedName::from_str(module);
        let file_name = format!("{}{}", scope.last()?, CAIRO_FILE_EXTENSION);
        self.paths.iter().map(|p| p.join(&file_name)).find(|path| path.exists())
    }
}

impl Default for ModuleReader {
    fn default() -> Self {
        Self {
            paths: std::env::var(LIBS_DIR_ENVVAR)
                .map(|p| vec![PathBuf::from(p)])
                .unwrap_or_default(),
            resolved_modules: Default::default(),
        }
    }
}

pub(crate) trait CodeReader {
    fn read(&self, module: &str) -> Result<(String, PathBuf)>;
}

impl<'a> CodeReader for &'a ModuleReader {
    /// Finds the module's file and read its content
    fn read(&self, module: &str) -> Result<(String, PathBuf)> {
        let file =
            self.find(module).ok_or_else(|| CairoError::ModuleNotFound(module.to_string()))?;
        Ok(fs::read_to_string(&file).map(|c| (c, file))?)
    }
}
