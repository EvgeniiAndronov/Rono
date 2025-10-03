use rono_lang::*;
use clap::{Arg, Command};
use std::fs;
use std::process;

fn main() {
    let matches = Command::new("rono")
        .version("0.1.0")
        .about("Rono Programming Language Compiler and Interpreter")
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("Run a Rono program in interpreted mode")
                .arg(
                    Arg::new("file")
                        .help("The input file to run")
                        .required(true)
                        .index(1),
                )
        )
        .subcommand(
            Command::new("compile")
                .about("Compile a Rono program to an executable")
                .arg(
                    Arg::new("file")
                        .help("The input file to compile")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output executable name")
                        .value_name("FILE"),
                )
                .arg(
                    Arg::new("target")
                        .short('t')
                        .long("target")
                        .help("Target platform for compilation")
                        .value_name("TARGET")
                        .value_parser(["x86_64-linux", "x86_64-windows", "x86_64-macos", "aarch64-linux", "aarch64-macos"]),
                )
                .arg(
                    Arg::new("optimize")
                        .short('O')
                        .long("optimize")
                        .help("Optimization level")
                        .value_name("LEVEL")
                        .value_parser(["none", "speed", "size"])
                        .default_value("none"),
                )
                .arg(
                    Arg::new("debug")
                        .short('g')
                        .long("debug")
                        .help("Include debug information")
                        .action(clap::ArgAction::SetTrue),
                )
        )
        // Legacy support for old CLI
        .arg(
            Arg::new("file")
                .help("The input file (legacy mode)")
                .index(1),
        )
        .arg(
            Arg::new("run")
                .short('r')
                .long("run")
                .help("Run the program (legacy mode)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let filename = sub_matches.get_one::<String>("file").unwrap();
            run_program(filename);
        }
        Some(("compile", sub_matches)) => {
            let filename = sub_matches.get_one::<String>("file").unwrap();
            let output = sub_matches.get_one::<String>("output");
            let target_str = sub_matches.get_one::<String>("target");
            let optimize_str = sub_matches.get_one::<String>("optimize").unwrap();
            let debug = sub_matches.get_flag("debug");
            
            compile_program(filename, output, target_str, optimize_str, debug);
        }
        _ => {
            // Legacy mode support
            if let Some(filename) = matches.get_one::<String>("file") {
                let run_mode = matches.get_flag("run");
                if run_mode {
                    run_program(filename);
                } else {
                    // Default to interpretation for legacy mode
                    run_program(filename);
                }
            } else {
                eprintln!("No input file specified. Use 'rono --help' for usage information.");
                process::exit(1);
            }
        }
    }
}

fn run_program(filename: &str) {
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

    // Interpretation
    let mut interpreter = interpreter::Interpreter::new();
    if let Err(e) = interpreter.execute(&ast) {
        eprintln!("Runtime error: {}", e);
        process::exit(1);
    }
}

fn compile_program(filename: &str, output: Option<&String>, target_str: Option<&String>, optimize_str: &str, debug: bool) {
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

    // Determine target
    let target = match target_str.map(|s| s.as_str()) {
        Some("x86_64-linux") => Target::X86_64Linux,
        Some("x86_64-windows") => Target::X86_64Windows,
        Some("x86_64-macos") => Target::X86_64MacOS,
        Some("aarch64-linux") => Target::Aarch64Linux,
        Some("aarch64-macos") => Target::Aarch64MacOS,
        None => detect_host_target(),
        Some(unknown) => {
            eprintln!("Unknown target: {}", unknown);
            process::exit(1);
        }
    };

    // Determine optimization level
    let opt_level = match optimize_str {
        "none" => OptLevel::None,
        "speed" => OptLevel::Speed,
        "size" => OptLevel::Size,
        _ => {
            eprintln!("Unknown optimization level: {}", optimize_str);
            process::exit(1);
        }
    };

    // Determine output filename
    let output_path = match output {
        Some(path) => path.clone(),
        None => {
            let base_name = std::path::Path::new(filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("program");
            
            match target {
                Target::X86_64Windows => format!("{}.exe", base_name),
                _ => base_name.to_string(),
            }
        }
    };

    // Create compiler and compile
    let mut compiler = match Compiler::new(target, opt_level, debug) {
        Ok(compiler) => compiler,
        Err(e) => {
            eprintln!("Failed to create compiler: {}", e);
            process::exit(1);
        }
    };

    match compiler.compile(&ast, &output_path) {
        Ok(()) => {
            if compiler.has_errors() {
                compiler.print_diagnostics();
                eprintln!("Compilation failed due to errors.");
                process::exit(1);
            } else {
                compiler.print_diagnostics(); // Print warnings and info
                println!("Compilation successful! Output: {}", output_path);
            }
        }
        Err(e) => {
            compiler.print_diagnostics();
            eprintln!("Compilation failed: {}", e);
            process::exit(1);
        }
    }
}