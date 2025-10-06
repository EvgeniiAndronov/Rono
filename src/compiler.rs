use crate::ast::Program;
use crate::semantic::SemanticAnalyzer;
use crate::ir_gen::IRGenerator;

use cranelift::prelude::settings::{self, Configurable};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;
use thiserror::Error;
use std::fs;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Semantic error at {location}: {message}")]
    Semantic {
        location: SourceLocation,
        message: String,
    },
    
    #[error("Semantic analysis error: {0}")]
    SemanticAnalysis(String),
    
    #[error("IR generation error: {0}")]
    IRGeneration(String),
    
    #[error("Code generation error: {0}")]
    CodeGeneration(String),
    
    #[error("Linker error: {0}")]
    Linker(String),
    
    #[error("Object write error: {0}")]
    ObjectWrite(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(#[from] crate::error::ChifError),
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

impl SourceLocation {
    pub fn new(file: String, line: usize, column: usize) -> Self {
        Self { file, line, column }
    }
    
    pub fn unknown() -> Self {
        Self {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Target {
    X86_64Linux,
    X86_64Windows,
    X86_64MacOS,
    Aarch64Linux,
    Aarch64MacOS,
}

impl Target {
    pub fn to_triple(&self) -> Triple {
        match self {
            Target::X86_64Linux => "x86_64-unknown-linux-gnu".parse().unwrap(),
            Target::X86_64Windows => "x86_64-pc-windows-msvc".parse().unwrap(),
            Target::X86_64MacOS => "x86_64-apple-darwin".parse().unwrap(),
            Target::Aarch64Linux => "aarch64-unknown-linux-gnu".parse().unwrap(),
            Target::Aarch64MacOS => "aarch64-apple-darwin".parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OptLevel {
    None,
    Speed,
    Size,
}

impl OptLevel {
    pub fn to_cranelift_opt_level(&self) -> settings::OptLevel {
        match self {
            OptLevel::None => settings::OptLevel::None,
            OptLevel::Speed => settings::OptLevel::Speed,
            OptLevel::Size => settings::OptLevel::SpeedAndSize,
        }
    }
}

pub struct Compiler {
    target: Target,
    optimization_level: OptLevel,
    debug_info: bool,
    diagnostics: Vec<CompilerDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct CompilerDiagnostic {
    pub level: DiagnosticLevel,
    pub location: SourceLocation,
    pub message: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
            DiagnosticLevel::Info => write!(f, "info"),
        }
    }
}

impl Compiler {
    pub fn new(target: Target, optimization_level: OptLevel, debug_info: bool) -> Result<Self, CompilerError> {
        let triple = target.to_triple();
        
        // Create ISA builder
        let mut builder = settings::builder();
        builder.set("opt_level", &optimization_level.to_cranelift_opt_level().to_string())
            .map_err(|e| CompilerError::CodeGeneration(format!("Failed to set optimization level: {}", e)))?;
        
        let flags = settings::Flags::new(builder);
        let isa = cranelift::codegen::isa::lookup(triple.clone())
            .map_err(|e| CompilerError::CodeGeneration(format!("Failed to lookup ISA: {}", e)))?
            .finish(flags)
            .map_err(|e| CompilerError::CodeGeneration(format!("Failed to create ISA: {}", e)))?;
        
        Ok(Self {
            target,
            optimization_level,
            debug_info,
            diagnostics: Vec::new(),
        })
    }
    
    pub fn compile(&mut self, ast: &Program, output_path: &str) -> Result<(), CompilerError> {
        println!("Starting compilation for target: {:?}", self.target);
        println!("Optimization level: {:?}", self.optimization_level);
        println!("Debug info: {}", self.debug_info);
        
        // 1. Semantic analysis
        println!("Performing semantic analysis...");
        let mut analyzer = SemanticAnalyzer::new();
        let analyzed_program = analyzer.analyze(ast)
            .map_err(|e| CompilerError::SemanticAnalysis(e.to_string()))?;
        
        // 2. Setup Cranelift
        println!("Setting up code generator...");
        let triple = self.target.to_triple();
        
        // Create ISA builder
        let mut builder = settings::builder();
        builder.set("opt_level", &self.optimization_level.to_cranelift_opt_level().to_string())
            .map_err(|e| CompilerError::CodeGeneration(format!("Failed to set optimization level: {}", e)))?;
            
        // Enable PIC for macOS ARM64
        #[cfg(target_os = "macos")]
        {
            builder.set("is_pic", "true")
                .map_err(|e| CompilerError::CodeGeneration(format!("Failed to set PIC: {}", e)))?;
        }
        
        let flags = settings::Flags::new(builder);
        let isa = cranelift::codegen::isa::lookup(triple.clone())
            .map_err(|e| CompilerError::CodeGeneration(format!("Failed to lookup ISA: {}", e)))?
            .finish(flags)
            .map_err(|e| CompilerError::CodeGeneration(format!("Failed to create ISA: {}", e)))?;
        
        let mut object_builder = ObjectBuilder::new(
            isa,
            "rono_program".to_string(),
            cranelift_module::default_libcall_names(),
        ).map_err(|e| CompilerError::CodeGeneration(format!("Failed to create object builder: {}", e)))?;
        
        // Enable PIC for macOS ARM64
        #[cfg(target_os = "macos")]
        {
            // This should help with text relocations
        }
        
        let module = ObjectModule::new(object_builder);
        
        // 3. IR generation
        println!("Generating IR...");
        let mut ir_generator = IRGenerator::new(module);
        ir_generator.generate(&analyzed_program)
            .map_err(|e| CompilerError::IRGeneration(e.to_string()))?;
        
        // 4. Code generation and object file creation
        println!("Generating object file...");
        let object_product = ir_generator.finalize().finish();
        
        // 5. Write object file
        let object_bytes = object_product.emit()
            .map_err(|e| CompilerError::ObjectWrite(e.to_string()))?;
        
        // Create build directory if it doesn't exist
        std::fs::create_dir_all("build")?;
        
        let object_path = format!("build/{}.o", output_path);
        let executable_path = format!("build/{}", output_path);
        
        fs::write(&object_path, object_bytes)?;
        
        println!("Object file created: {}", object_path);
        
        // 6. Link to create executable
        println!("Linking executable...");
        self.link_executable(&object_path, &executable_path)?;
        
        Ok(())
    }
    
    pub fn compile_to_object(&mut self, _ast: &Program) -> Result<Vec<u8>, CompilerError> {
        // TODO: Implement object file generation
        Err(CompilerError::CodeGeneration("Object compilation not yet implemented".to_string()))
    }
    
    fn link_executable(&self, object_file: &str, output_path: &str) -> Result<(), CompilerError> {
        use std::process::Command;
        
        // First, compile runtime library if needed
        let runtime_obj = "build/runtime.o";
        if !std::path::Path::new(runtime_obj).exists() {
            println!("Compiling runtime library...");
            std::fs::create_dir_all("build")?;
            let mut compile_cmd = Command::new("cc");
            compile_cmd.arg("-c")
                      .arg("src/runtime.c")
                      .arg("-o")
                      .arg(runtime_obj);
            
            let compile_output = compile_cmd.output()
                .map_err(|e| CompilerError::CodeGeneration(format!("Failed to compile runtime: {}", e)))?;
            
            if !compile_output.status.success() {
                let stderr = String::from_utf8_lossy(&compile_output.stderr);
                return Err(CompilerError::CodeGeneration(format!("Runtime compilation failed: {}", stderr)));
            }
        }
        
        // Use system linker to create executable
        let mut cmd = Command::new("cc"); // Use system C compiler as linker
        cmd.arg("-o").arg(output_path);
        cmd.arg(object_file);
        cmd.arg(runtime_obj); // Link with runtime
        
        // Add platform-specific flags
        #[cfg(target_os = "macos")]
        {
            cmd.arg("-Wl,-no_pie"); // Disable PIE to avoid text relocations
        }
        
        // Add system libraries
        #[cfg(target_os = "macos")]
        {
            cmd.arg("-lSystem");
            cmd.arg("-lcurl"); // Link with libcurl
        }
        #[cfg(target_os = "linux")]
        {
            cmd.arg("-lc");
            cmd.arg("-lcurl"); // Link with libcurl
        }
        #[cfg(target_os = "windows")]
        {
            // Windows linking would be different
            return Err(CompilerError::CodeGeneration("Windows linking not yet implemented".to_string()));
        }
        
        let output = cmd.output()
            .map_err(|e| CompilerError::CodeGeneration(format!("Failed to run linker: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CompilerError::CodeGeneration(format!("Linking failed: {}", stderr)));
        }
        
        println!("Executable created: {}", output_path);
        Ok(())
    }

    pub fn add_diagnostic(&mut self, diagnostic: CompilerDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
    
    pub fn add_error(&mut self, location: SourceLocation, message: String, code: Option<String>) {
        self.add_diagnostic(CompilerDiagnostic {
            level: DiagnosticLevel::Error,
            location,
            message,
            code,
        });
    }
    
    pub fn add_warning(&mut self, location: SourceLocation, message: String, code: Option<String>) {
        self.add_diagnostic(CompilerDiagnostic {
            level: DiagnosticLevel::Warning,
            location,
            message,
            code,
        });
    }
    
    pub fn add_info(&mut self, location: SourceLocation, message: String, code: Option<String>) {
        self.add_diagnostic(CompilerDiagnostic {
            level: DiagnosticLevel::Info,
            location,
            message,
            code,
        });
    }
    
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| matches!(d.level, DiagnosticLevel::Error))
    }
    
    pub fn print_diagnostics(&self) {
        for diagnostic in &self.diagnostics {
            eprintln!("{}: {}: {}", diagnostic.level, diagnostic.location, diagnostic.message);
            if let Some(code) = &diagnostic.code {
                eprintln!("  Code: {}", code);
            }
        }
    }
    
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }
}

// Helper function to detect host target
pub fn detect_host_target() -> Target {
    let triple = Triple::host();
    
    match (triple.architecture, triple.operating_system) {
        (target_lexicon::Architecture::X86_64, target_lexicon::OperatingSystem::Linux) => Target::X86_64Linux,
        (target_lexicon::Architecture::X86_64, target_lexicon::OperatingSystem::Windows) => Target::X86_64Windows,
        (target_lexicon::Architecture::X86_64, target_lexicon::OperatingSystem::Darwin) => Target::X86_64MacOS,
        (target_lexicon::Architecture::Aarch64(_), target_lexicon::OperatingSystem::Linux) => Target::Aarch64Linux,
        (target_lexicon::Architecture::Aarch64(_), target_lexicon::OperatingSystem::Darwin) => Target::Aarch64MacOS,
        _ => {
            eprintln!("Warning: Unsupported target architecture, defaulting to x86_64 Linux");
            Target::X86_64Linux
        }
    }
}