use std::path::PathBuf;
use std::{fs, io};

use miette::Diagnostic;
use thiserror::Error;
use tracing::info;

use crate::parser::lexer::{Lexer, LexingError, Token};
use crate::span::Spanned;

#[derive(Debug, Error, Diagnostic)]
pub enum DriverError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub struct Driver {
    input_path: PathBuf,
    source: String,
}

impl Driver {
    pub fn new(input_path: PathBuf) -> Result<Self, DriverError> {
        info!("read file '{}'", input_path.display());
        let source = fs::read_to_string(&input_path).map_err(DriverError::Io)?;
        Ok(Self { input_path, source })
    }

    pub fn lex<'a>(&'a self) -> Vec<Result<Spanned<Token<'a>>, LexingError>> {
        info!("lex file '{}'", self.input_path.display());
        let lexer = Lexer::new(&self.source);
        lexer.collect()
    }
}
