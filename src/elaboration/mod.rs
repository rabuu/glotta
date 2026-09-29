pub mod loop_labeling;
pub mod name_resolution;

use miette::Diagnostic;
use thiserror::Error;

use crate::ast;
use crate::span::Span;

use loop_labeling::label_loops;
use name_resolution::resolve_names;

type Result<T> = std::result::Result<T, ElaborationError>;

#[derive(Debug, Error, Diagnostic)]
pub enum ElaborationError {
    #[error("The function `{name}` is declared multiple times.")]
    DuplicateFunction {
        name: String,
        #[label]
        span: Span,
    },

    #[error("The parameter `{name}` is declared multiple times.")]
    DuplicateParameter {
        name: String,
        #[label]
        span: Span,
    },

    #[error("The name `{name}` is not bound.")]
    NameNotBound {
        name: String,
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
    resolve_names(program)?;
    label_loops(program)?;
    Ok(())
}
