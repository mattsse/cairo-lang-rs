use crate::lexer::CairoLexerError;
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
}
