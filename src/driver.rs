use std::path::PathBuf;
use std::process::Command;
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

    #[error("Running `nasm -f elf64 {assembly_path} -o {object_path}` failed.")]
    Nasm {
        assembly_path: PathBuf,
        object_path: PathBuf,
    },

    #[error("Running `cc {object_path} -o {executable_path}` failed.")]
    Cc {
        object_path: PathBuf,
        executable_path: PathBuf,
    },

    #[error("The input path '{path}' is invalid.")]
    InvalidInputPath { path: PathBuf },

    #[error("The implicit output file '{path}' would overwrite the input file.")]
    ImplicitOutputFileOverwritesInputFile { path: PathBuf },
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
        let asm = codegen::codegen_program(&ast);
        Ok(asm)
    }

    pub fn emit_assembly_to_file(&self, assembly_path: Option<PathBuf>) -> Result<PathBuf> {
        let assembly_path = match assembly_path {
            Some(path) => path,
            None => self.default_output_path(OutputFormat::Assembly)?,
        };

        let asm = self.codegen()?;

        info!(
            "emit assembly from '{}' to '{}'",
            self.input_path.display(),
            assembly_path.display()
        );

        let file = fs::File::create(&assembly_path)?;
        let writer = io::BufWriter::new(file);
        let emitter = Emitter::new(writer);
        emitter.emit_program(&asm)?;

        Ok(assembly_path)
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

    pub fn generate_object_file(
        &self,
        object_path: Option<PathBuf>,
        keep_build_artifacts: bool,
    ) -> Result<PathBuf> {
        let object_path = match object_path {
            Some(path) => path,
            None => self.default_output_path(OutputFormat::Object)?,
        };

        let assembly_path = self.emit_assembly_to_file(None)?;

        info!(
            "assemble '{}' to '{}' with `nasm`",
            assembly_path.display(),
            object_path.display()
        );

        let status = Command::new("nasm")
            .args(["-f", "elf64"])
            .arg(&assembly_path)
            .arg("-o")
            .arg(&object_path)
            .status()?;

        if !status.success() {
            return Err(DriverError::Nasm {
                assembly_path,
                object_path,
            });
        }

        if !keep_build_artifacts {
            fs::remove_file(&assembly_path)?;
        }

        Ok(object_path)
    }

    pub fn compile_to_executable_file(
        &self,
        executable_path: Option<PathBuf>,
        keep_build_artifacts: bool,
    ) -> Result<PathBuf> {
        let executable_path = match executable_path {
            Some(path) => path,
            None => self.default_output_path(OutputFormat::Executable)?,
        };

        let object_path = self.generate_object_file(None, keep_build_artifacts)?;

        info!(
            "link '{}' to '{}' with `cc`",
            object_path.display(),
            executable_path.display()
        );

        let status = Command::new("cc")
            .arg(&object_path)
            .arg("-o")
            .arg(&executable_path)
            .status()?;

        if !status.success() {
            return Err(DriverError::Cc {
                object_path,
                executable_path,
            });
        }

        if !keep_build_artifacts {
            fs::remove_file(&object_path)?;
        }

        Ok(executable_path)
    }

    pub fn default_output_path(&self, format: OutputFormat) -> Result<PathBuf> {
        let mut path = self.input_path.clone();
        path.set_extension(format.extension());

        if path.exists() && path == self.input_path {
            return Err(DriverError::ImplicitOutputFileOverwritesInputFile { path });
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
    Executable,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Assembly => "asm",
            OutputFormat::Object => "o",
            OutputFormat::Executable => "",
        }
    }
}
