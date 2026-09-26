pub mod name_resolution;

use miette::Diagnostic;
use thiserror::Error;

use crate::ast;
use crate::span::Span;

use name_resolution::NameResolver;

type Result<T> = std::result::Result<T, ElaborationError>;

#[derive(Debug, Error, Diagnostic)]
pub enum ElaborationError {
    #[error("The function name `{name}` is declared multiple times.")]
    DuplicateFunctionName {
        name: String,
        #[label]
        span: Span,
    },

    #[error("The variable `{variable}` is not bound.")]
    VariableNotBound {
        variable: String,
        #[label]
        span: Span,
    },

    #[error("The function `{function}` is not bound.")]
    FunctionNotBound {
        function: String,
        #[label]
        span: Span,
    },
}

pub fn elaborate(program: &mut ast::Program) -> Result<()> {
    let mut name_resolver = NameResolver::new();
    name_resolver.resolve(program)?;

    Ok(())
}
