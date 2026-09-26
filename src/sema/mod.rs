pub mod name_resolution;

use miette::Diagnostic;
use thiserror::Error;

use crate::span::Span;

type Result<T> = std::result::Result<T, SemanticError>;

#[derive(Debug, Error, Diagnostic)]
pub enum SemanticError {
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
