pub use crate::compiler::{data::DebugInfo, module_reader::ModuleReader};
use crate::{
    compiler::sema::{PreprocessedProgram, ScopedName},
    error::Result,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// compiler builtins
mod builtins;
pub mod constants;
mod data;
mod instruction;
mod module_reader;
mod program;
pub use program::Program;
mod sema;
use crate::compiler::constants::{START_CODE, START_FILE_NAME};
pub use sema::{
    ast::{VResult, Visitable, Visitor},
    passes::PassManager,
};

/// Utility struct to compile a list of cairo files
#[derive(Debug, Clone)]
pub struct CairoCompiler {
    /// files to compile
    files: Vec<PathBuf>,
    /// whether to include debug info
    debug_info: bool,

    main_scope: Option<ScopedName>,
}

impl CairoCompiler {
    pub fn compile(&self) -> Result<Program> {
        todo!()
    }
}

/// Compiles a list of cairo files
pub fn compile_cairo<I, P>(
    files: I,
    _debug_info: bool,
    add_start: bool,
    pass_manager: impl Into<PassManager>,
    _module_reader: &mut ModuleReader,
    main_scope: Option<ScopedName>,
) -> Result<Program>
where
    I: IntoIterator<Item = P>,
    P: Into<PathBuf>,
{
    let mut codes = read_files(files)?;
    if add_start {
        codes.insert(0, start_code());
    }

    let mut debug_info = DebugInfo::default();
    if let Some((content, file)) =
        codes.get(0).filter(|(_, file)| file == Path::new(START_FILE_NAME))
    {
        debug_info.file_contents.insert(file.clone(), content.clone());
    }

    let mut pass_manager = pass_manager.into();
    let main_scope = main_scope.unwrap_or_else(ScopedName::main_scope);

    // preprocess the cairo program
    let mut prg = PreprocessedProgram::new(main_scope, codes);
    // execute all compiler passes
    pass_manager.run_on(&mut prg)?;
    // TODO assemble the cairo program
    todo!()
}

/// Reads all given files and returns them zipped with their content
fn read_files(
    files: impl IntoIterator<Item = impl Into<PathBuf>>,
) -> io::Result<Vec<(String, PathBuf)>> {
    files
        .into_iter()
        .map(Into::into)
        .map(|file| fs::read_to_string(&file).map(|c| (c, file)))
        .collect()
}

fn start_code() -> (String, PathBuf) {
    (START_CODE.to_string(), START_FILE_NAME.into())
}
