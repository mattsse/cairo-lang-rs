use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModuleReader {
    /// where to look for paths
    paths: Vec<PathBuf>,
}
