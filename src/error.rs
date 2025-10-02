use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChifError {
    #[error("Lexer error at line {line}, column {column}: {message}")]
    LexerError {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Parser error: {message}")]
    ParserError { message: String },
    
    #[error("Type error: {message}")]
    TypeError { message: String },
    
    #[error("Runtime error: {message}")]
    RuntimeError { message: String },
    
    #[error("Variable '{name}' not found")]
    VariableNotFound { name: String },
    
    #[error("Function '{name}' not found")]
    FunctionNotFound { name: String },
    
    #[error("Index out of bounds: {index}")]
    IndexOutOfBounds { index: usize },
    
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
    
    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },
    
    #[error("Return value")]
    Return(crate::types::ChifValue),
}

pub type Result<T> = std::result::Result<T, ChifError>;