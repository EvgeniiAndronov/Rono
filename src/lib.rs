pub mod lexer;
pub mod parser;
pub mod ast;
pub mod interpreter;
pub mod types;
pub mod error;
pub mod compiler;
pub mod semantic;
pub mod ir_gen;

#[cfg(test)]
mod semantic_test;

pub use error::{ChifError, Result};
pub use lexer::Lexer;
pub use parser::Parser;
pub use interpreter::Interpreter;
pub use ast::Program;
pub use types::{ChifType, ChifValue};
pub use compiler::{Compiler, CompilerError, Target, OptLevel, detect_host_target};
pub use semantic::{SemanticAnalyzer, SemanticError, AnalyzedProgram};
pub use ir_gen::{IRGenerator, IRError};