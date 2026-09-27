pub mod loop_labeling;
pub mod name_resolution;

use miette::Diagnostic;
use thiserror::Error;

use crate::ast;
use crate::span::Span;

use loop_labeling::LoopLabeler;
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

    #[error("The `break` expression is not enclosed by a loop.")]
    OrphanedBreak {
        #[label]
        span: Span,
    },

    #[error("The `continue` expression is not enclosed by a loop.")]
    OrphanedContinue {
        #[label]
        span: Span,
    },
}

pub fn elaborate(program: &mut ast::Program) -> Result<()> {
    let mut name_resolver = NameResolver::new();
    name_resolver.resolve(program)?;

    let mut loop_labeler = LoopLabeler::new();
    loop_labeler.label(program)?;

    Ok(())
}
