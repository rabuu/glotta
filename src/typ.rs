use std::{fmt, iter};

#[derive(Debug, PartialEq)]
pub enum Type {
    Int,
    Function(FunctionType),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Function(function_type) => write!(f, "{function_type}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionType {
    // for now, every parameter must be Int
    pub parameters: usize,
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FunctionType { parameters } = self;

        let parameters: Vec<&str> = iter::repeat_n("Int", *parameters).collect();
        let parameters = parameters.join(", ");

        write!(f, "({parameters}) -> Int")
    }
}
