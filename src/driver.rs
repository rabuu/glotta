use std::path::PathBuf;
use std::{fs, io};

use miette::Diagnostic;
use thiserror::Error;
use tracing::info;

use crate::ast;
use crate::codegen::emitter::Emitter;
use crate::codegen::{self, asm};
use crate::parser::{Lexer, Parser, ParsingError};

type Result<T> = std::result::Result<T, DriverError>;

#[derive(Debug, Error, Diagnostic)]
pub enum DriverError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Parsing(#[from] ParsingError),

    #[error("The input path '{path}' is invalid.")]
    InvalidInputPath { path: PathBuf },

    #[error("The output path and input path are the same: '{path}'")]
    OutputPathIsInputPath { path: PathBuf },
}

pub struct Driver {
    input_path: PathBuf,
    source: String,
}

impl Driver {
    pub fn new(input_path: PathBuf) -> Result<Self> {
        if !input_path.is_file() || !input_path.file_name().is_some() {
            return Err(DriverError::InvalidInputPath { path: input_path });
        }

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

    pub fn emit_assembly_to_file<P>(&self, path: Option<P>) -> Result<()>
    where
        P: Into<PathBuf>,
    {
        let asm = self.codegen()?;

        let path = match path {
            Some(path) => path.into(),
            None => self.default_output_path(OutputFormat::Elf)?,
        };

        info!(
            "emit assembly from '{}' to file '{}'",
            self.input_path.display(),
            path.display()
        );

        let file = fs::File::create(path)?;
        let writer = io::BufWriter::new(file);
        let emitter = Emitter::new(writer);
        emitter.emit_program(&asm)?;

        Ok(())
    }

    pub fn emit_assembly_to_stdout(&self) -> Result<()> {
        let asm = self.codegen()?;

        info!(
            "emit assembly from '{}' to stdout",
            self.input_path.display()
        );

        let stdout = io::stdout().lock();
        let writer = io::BufWriter::new(stdout);
        let emitter = Emitter::new(writer);
        emitter.emit_program(&asm)?;

        Ok(())
    }

    pub fn default_output_path(&self, format: OutputFormat) -> Result<PathBuf> {
        let mut path = self.input_path.clone();
        path.set_extension(format.extension());

        if path == self.input_path {
            return Err(DriverError::OutputPathIsInputPath { path });
        }

        Ok(path)
    }

    pub fn to_report(&self, error: DriverError) -> miette::Report {
        let error: miette::Error = error.into();
        let source = miette::NamedSource::new(
            self.input_path.display().to_string(),
            self.source.to_string(),
        );
        error.with_source_code(source)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Assembly,
    Object,
    Elf,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Assembly => "asm",
            OutputFormat::Object => "o",
            OutputFormat::Elf => "",
        }
    }
}
