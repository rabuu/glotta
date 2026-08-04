use std::path::PathBuf;
use std::{fs, io};

use miette::Diagnostic;
use thiserror::Error;
use tracing::info;

use crate::ast;
use crate::parser::lexer::Lexer;
use crate::parser::parser::{Parser, ParsingError};

type Result<T> = std::result::Result<T, DriverError>;

#[derive(Debug, Error, Diagnostic)]
#[error(transparent)]
pub enum DriverError {
    Io(#[from] io::Error),

    #[diagnostic(transparent)]
    Parsing(#[from] ParsingError),
}

pub struct Driver {
    input_path: PathBuf,
    source: String,
}

impl Driver {
    pub fn new(input_path: PathBuf) -> Result<Self> {
        info!("read file '{}'", input_path.display());
        let source = fs::read_to_string(&input_path).map_err(DriverError::Io)?;
        Ok(Self { input_path, source })
    }

    pub fn source<'src>(&'src self) -> &'src str {
        &self.source
    }

    pub fn lexer<'src>(&'src self) -> Lexer<'src> {
        Lexer::new(&self.source)
    }

    pub fn parse(&self) -> Result<ast::Program> {
        info!("parse file '{}'", self.input_path.display());
        let lexer = self.lexer();
        let parser = Parser::new(lexer);
        parser.parse_program().map_err(DriverError::Parsing)
    }
}
