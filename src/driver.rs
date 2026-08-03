use std::fs;
use std::path::PathBuf;

use tracing::info;

use crate::Spanned;
use crate::parser::lexer::{Lexer, LexingError, Token};

pub struct Driver {
    input_path: PathBuf,
    source: String,
}

impl Driver {
    pub fn new(input_path: PathBuf) -> Option<Self> {
        let source = fs::read_to_string(&input_path).ok()?;

        info!("read file '{}'", input_path.display());

        Some(Self { input_path, source })
    }

    pub fn lex<'a>(&'a self) -> Vec<Result<Spanned<Token<'a>>, LexingError>> {
        let lexer = Lexer::new(&self.source);
        lexer.collect()
    }
}
