use std::path::PathBuf;
use std::{fs, io};

use miette::Diagnostic;
use thiserror::Error;
use tracing::info;

use crate::ast;
use crate::codegen::{self, asm};
use crate::parser::{Lexer, Parser, ParsingError};

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

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn lexer<'src>(&'src self) -> Lexer<'src> {
        Lexer::new(&self.source)
    }

    pub fn parse(&self) -> Result<ast::Program> {
        info!("parse '{}'", self.input_path.display());
        let lexer = self.lexer();
        let parser = Parser::new(lexer);
        parser.parse_program().map_err(DriverError::Parsing)
    }

    pub fn codegen(&self) -> Result<asm::Program> {
        let ast = self.parse()?;
        info!("codegen '{}'", self.input_path.display());
        let asm = codegen::codegen_program(&ast).unwrap();
        Ok(asm)
    }

    pub fn emit_assembly_to_file(&self, output: Option<PathBuf>) -> Result<()> {
        let asm = self.codegen()?;
        let output = output.unwrap_or_else(|| {
            let mut output = self.input_path.clone();
            let set_extension = output.set_extension("s");
            assert!(set_extension);
            output
        });
        std::fs::write(output, asm.to_string()).map_err(DriverError::Io)
    }

    pub fn print_error(&self, error: DriverError) {
        let error: miette::Error = error.into();
        let source = miette::NamedSource::new(
            self.input_path.display().to_string(),
            self.source.to_string(),
        );
        let report: miette::Report = error.with_source_code(source);
        eprintln!("{report:?}");
    }
}
