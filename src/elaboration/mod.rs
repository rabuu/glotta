pub mod loop_labeling;
pub mod name_resolution;
pub mod typechecking;

use std::collections::HashMap;

use miette::Diagnostic;
use thiserror::Error;

use crate::ast;
use crate::span::Span;
use crate::typ::Type;

use loop_labeling::label_loops;
use name_resolution::resolve_names;
use typechecking::typecheck;

pub type Id = usize;
pub type SymbolTable = HashMap<Id, Type>;

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

    #[error("Expected {expected} arguments for builtin call, but got {got}.")]
    BuiltinArityMismatch {
        expected: usize,
        got: usize,
        #[label]
        span: Span,
    },

    #[error("Expected {expected} arguments for function call, but got {got}.")]
    FunctionArityMismatch {
        expected: usize,
        got: usize,
        #[label]
        span: Span,
    },

    #[error("The name `{name}` is not a function.")]
    NotAFunction {
        name: String,
        #[label]
        span: Span,
    },

    #[error("The name `{name}` is not a variable.")]
    NotAVariable {
        name: String,
        #[label]
        span: Span,
    },
}

pub fn elaborate(program: &mut ast::Program) -> Result<SymbolTable> {
    resolve_names(program)?;
    label_loops(program)?;
    let symbol_table = typecheck(program)?;

    Ok(symbol_table)
}
