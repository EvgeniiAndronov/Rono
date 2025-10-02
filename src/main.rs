pub mod lexer;
pub mod parser;
pub mod ast;
pub mod interpreter;
pub mod types;
pub mod error;

use clap::{Arg, Command};
use std::fs;
use std::process;

fn main() {
    let matches = Command::new("rono")
        .version("0.1.0")
        .about("Rono Programming Language Compiler")
        .arg(
            Arg::new("file")
                .help("The input file to compile")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("run")
                .short('r')
                .long("run")
                .help("Run the program after compilation")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let filename = matches.get_one::<String>("file").unwrap();
    let run_mode = matches.get_flag("run");

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    // Lexical analysis
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Lexer error: {}", e);
            process::exit(1);
        }
    };

    // Parsing
    let mut parser = parser::Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            process::exit(1);
        }
    };

    if run_mode {
        // Interpretation
        let mut interpreter = interpreter::Interpreter::new();
        if let Err(e) = interpreter.execute(&ast) {
            eprintln!("Runtime error: {}", e);
            process::exit(1);
        }
    } else {
        println!("Compilation successful!");
    }
}