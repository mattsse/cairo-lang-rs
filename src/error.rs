use crate::parser::lexer::CairoLexerError;
use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, CairoError>;

/// Common error types used across the repo
#[derive(Debug, Error)]
pub enum CairoError {
    #[error(transparent)]
    Lexer(#[from] CairoLexerError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Circular imports: {0:?}")]
    CircularDependencies(Vec<String>),
    #[error("Could not find module: {0}")]
    ModuleNotFound(String),
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    InvalidImport(String),
    #[error("{0}")]
    Identifier(String),
    #[error("{0}")]
    Preprocess(String),
}

impl CairoError {
    pub fn msg(msg: impl Into<String>) -> Self {
        CairoError::Message(msg.into())
    }
}
