use crate::ast::*;
use crate::types::{ChifType, ChifValue};
use crate::compiler::SourceLocation;
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("Type mismatch at {location}: expected {expected:?}, found {found:?}")]
    TypeMismatch {
        location: SourceLocation,
        expected: ChifType,
        found: ChifType,
    },
    
    #[error("Undefined symbol '{symbol}' at {location}")]
    UndefinedSymbol {
        symbol: String,
        location: SourceLocation,
    },
    
    #[error("Symbol '{symbol}' already defined at {location}")]
    SymbolAlreadyDefined {
        symbol: String,
        location: SourceLocation,
    },
    
    #[error("Invalid operation at {location}: {message}")]
    InvalidOperation {
        location: SourceLocation,
        message: String,
    },
    
    #[error("Break statement outside of loop")]
    InvalidBreak,
    
    #[error("Continue statement outside of loop")]
    InvalidContinue,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub location: SourceLocation,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub enum SymbolType {
    Variable(ChifType),
    Function(FunctionSignature),
    Struct(StructDefinition),
    Module(ModuleInfo),
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: ChifType,
    pub is_mutating: bool,  // Новое поле для отслеживания мутирующих методов
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
}



#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
}

impl Scope {
    pub fn new(parent: Option<usize>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
        }
    }
    
    pub fn define_symbol(&mut self, symbol: Symbol) -> Result<(), SemanticError> {
        if self.symbols.contains_key(&symbol.name) {
            return Err(SemanticError::SymbolAlreadyDefined {
                symbol: symbol.name.clone(),
                location: symbol.location.clone(),
            });
        }
        
        self.symbols.insert(symbol.name.clone(), symbol);
        Ok(())
    }
    
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    pub scopes: Vec<Scope>,
    pub current_scope: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        let global_scope = Scope::new(None);
        Self {
            scopes: vec![global_scope],
            current_scope: 0,
        }
    }
    
    pub fn push_scope(&mut self) {
        let parent = Some(self.current_scope);
        let new_scope = Scope::new(parent);
        self.scopes.push(new_scope);
        self.current_scope = self.scopes.len() - 1;
    }
    
    pub fn pop_scope(&mut self) -> Result<(), SemanticError> {
        if self.current_scope == 0 {
            return Err(SemanticError::InvalidOperation {
                location: SourceLocation::unknown(),
                message: "Cannot pop global scope".to_string(),
            });
        }
        
        let current = &self.scopes[self.current_scope];
        if let Some(parent) = current.parent {
            self.current_scope = parent;
        }
        
        Ok(())
    }
    
    pub fn define_symbol(&mut self, symbol: Symbol) -> Result<(), SemanticError> {
        self.scopes[self.current_scope].define_symbol(symbol)
    }
    
    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        let mut current_scope = self.current_scope;
        
        loop {
            if let Some(symbol) = self.scopes[current_scope].lookup_symbol(name) {
                return Some(symbol);
            }
            
            if let Some(parent) = self.scopes[current_scope].parent {
                current_scope = parent;
            } else {
                break;
            }
        }
        
        None
    }
}

pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable,
    pub in_loop: bool,
    pub current_function_return_type: Option<ChifType>,
    pub modules: HashMap<String, ModuleInfo>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub functions: HashMap<String, FunctionSignature>,
    pub structs: HashMap<String, StructDefinition>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            in_loop: false,
            current_function_return_type: None,
            modules: HashMap::new(),
        }
    }
    
    pub fn check_types(&mut self, program: &Program) -> Result<(), SemanticError> {
        for item in &program.items {
            self.check_item_types(item)?;
        }
        Ok(())
    }
    
    fn check_item_types(&mut self, item: &Item) -> Result<(), SemanticError> {
        match item {
            Item::Function(func) => {
                self.symbol_table.push_scope();
                
                // Set current function return type for validation
                let old_return_type = self.current_function_return_type.clone();
                self.current_function_return_type = func.return_type.clone();
                
                // Add parameters to scope
                for param in &func.params {
                    let symbol = Symbol {
                        name: param.name.clone(),
                        symbol_type: SymbolType::Variable(param.param_type.clone()),
                        location: SourceLocation::unknown(),
                        is_mutable: false,
                    };
                    self.symbol_table.define_symbol(symbol)?;
                }
                
                // Check function body types
                self.check_block_types(&func.body, &func.return_type)?;
                
                // Validate that all code paths return a value if needed
                // For main function, we allow implicit nil return
                if let Some(return_type) = &func.return_type {
                    if *return_type != ChifType::Nil && !func.is_main && !self.block_always_returns(&func.body) {
                        return Err(SemanticError::InvalidOperation {
                            location: SourceLocation::unknown(),
                            message: format!(
                                "Function '{}' must return a value of type {:?} in all code paths",
                                func.name, return_type
                            ),
                        });
                    }
                }
                
                // Restore previous function return type
                self.current_function_return_type = old_return_type;
                
                self.symbol_table.pop_scope()?;
            }
            Item::Struct(_struct_def) => {
                // Struct definitions are already handled in collect_definitions
                // No need to redefine them here
            }
            Item::StructImpl(impl_block) => {
                for method in &impl_block.methods {
                    self.check_item_types(&Item::Function(method.clone()))?;
                }
            }
            Item::Import(_) => {
                // Import type checking would be done during module resolution
            }
        }
        Ok(())
    }
    
    fn check_block_types(&mut self, block: &Block, expected_return_type: &Option<ChifType>) -> Result<(), SemanticError> {
        for statement in &block.statements {
            self.check_statement_types(statement, expected_return_type)?;
        }
        Ok(())
    }
    
    fn check_statement_types(&mut self, statement: &Statement, expected_return_type: &Option<ChifType>) -> Result<(), SemanticError> {
        match statement {
            Statement::VarDecl(var_decl) => {
                if let Some(expr) = &var_decl.value {
                    let expr_type = self.analyze_expression(expr)?;
                    if !self.types_compatible(&var_decl.var_type, &expr_type) {
                        return Err(SemanticError::TypeMismatch {
                            location: SourceLocation::unknown(),
                            expected: var_decl.var_type.clone(),
                            found: expr_type,
                        });
                    }
                }
                
                let symbol = Symbol {
                    name: var_decl.name.clone(),
                    symbol_type: SymbolType::Variable(var_decl.var_type.clone()),
                    location: SourceLocation::unknown(),
                    is_mutable: var_decl.is_mutable,
                };
                self.symbol_table.define_symbol(symbol)?;
            }
            Statement::Assignment(assignment) => {
                let target_type = self.analyze_expression(&assignment.target)?;
                let value_type = self.analyze_expression(&assignment.value)?;
                
                if !self.types_compatible(&target_type, &value_type) {
                    return Err(SemanticError::TypeMismatch {
                        location: SourceLocation::unknown(),
                        expected: target_type,
                        found: value_type,
                    });
                }
            }
            Statement::Return(expr) => {
                // We're always in a function context during check_statement_types
                // The current_function_return_type is set in analyze_item
                
                if let Some(expr) = expr {
                    let return_type = self.analyze_expression(expr)?;
                    if let Some(expected) = expected_return_type {
                        if !self.types_compatible(expected, &return_type) {
                            return Err(SemanticError::TypeMismatch {
                                location: SourceLocation::unknown(),
                                expected: expected.clone(),
                                found: return_type,
                            });
                        }
                    }
                } else {
                    // Empty return
                    if let Some(expected) = expected_return_type {
                        if *expected != ChifType::Nil {
                            return Err(SemanticError::TypeMismatch {
                                location: SourceLocation::unknown(),
                                expected: expected.clone(),
                                found: ChifType::Nil,
                            });
                        }
                    }
                }
            }
            Statement::If(if_stmt) => {
                let condition_type = self.analyze_expression(&if_stmt.condition)?;
                if condition_type != ChifType::Bool {
                    return Err(SemanticError::TypeMismatch {
                        location: SourceLocation::unknown(),
                        expected: ChifType::Bool,
                        found: condition_type,
                    });
                }
                
                self.check_block_types(&if_stmt.then_block, expected_return_type)?;
                if let Some(else_block) = &if_stmt.else_block {
                    self.check_block_types(else_block, expected_return_type)?;
                }
            }
            Statement::While(while_stmt) => {
                let condition_type = self.analyze_expression(&while_stmt.condition)?;
                if condition_type != ChifType::Bool {
                    return Err(SemanticError::TypeMismatch {
                        location: SourceLocation::unknown(),
                        expected: ChifType::Bool,
                        found: condition_type,
                    });
                }
                
                // Enter loop context
                let old_in_loop = self.in_loop;
                self.in_loop = true;
                
                self.check_block_types(&while_stmt.body, expected_return_type)?;
                
                // Restore loop context
                self.in_loop = old_in_loop;
            }
            Statement::For(for_stmt) => {
                self.symbol_table.push_scope();
                
                if let Some(init) = &for_stmt.init {
                    self.check_statement_types(init, expected_return_type)?;
                }
                
                if let Some(condition) = &for_stmt.condition {
                    let condition_type = self.analyze_expression(condition)?;
                    if condition_type != ChifType::Bool {
                        return Err(SemanticError::TypeMismatch {
                            location: SourceLocation::unknown(),
                            expected: ChifType::Bool,
                            found: condition_type,
                        });
                    }
                }
                
                if let Some(update) = &for_stmt.update {
                    self.analyze_statement(update)?;
                }
                
                // Enter loop context
                let old_in_loop = self.in_loop;
                self.in_loop = true;
                
                self.check_block_types(&for_stmt.body, expected_return_type)?;
                
                // Restore loop context
                self.in_loop = old_in_loop;
                
                self.symbol_table.pop_scope()?;
            }
            Statement::Switch(switch_stmt) => {
                let switch_type = self.analyze_expression(&switch_stmt.expr)?;
                
                for case in &switch_stmt.cases {
                    let case_type = self.analyze_expression(&case.value)?;
                    if !self.types_compatible(&switch_type, &case_type) {
                        return Err(SemanticError::TypeMismatch {
                            location: SourceLocation::unknown(),
                            expected: switch_type.clone(),
                            found: case_type,
                        });
                    }
                    self.check_block_types(&case.body, expected_return_type)?;
                }
                
                if let Some(default_case) = &switch_stmt.default_case {
                    self.check_block_types(default_case, expected_return_type)?;
                }
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
            Statement::Break => {
                // Check if we're in a loop context
                if !self.in_loop {
                    return Err(SemanticError::InvalidBreak);
                }
            }
            Statement::Continue => {
                // Check if we're in a loop context
                if !self.in_loop {
                    return Err(SemanticError::InvalidContinue);
                }
            }
        }
        
        Ok(())
    }
    
    fn types_compatible(&self, expected: &ChifType, actual: &ChifType) -> bool {
        match (expected, actual) {
            // Exact matches
            (ChifType::Int, ChifType::Int) => true,
            (ChifType::Float, ChifType::Float) => true,
            (ChifType::Str, ChifType::Str) => true,
            (ChifType::Bool, ChifType::Bool) => true,
            (ChifType::Nil, ChifType::Nil) => true,
            
            // Numeric conversions
            (ChifType::Float, ChifType::Int) => true, // Int can be promoted to Float
            
            // Array/List compatibility
            (ChifType::Array(expected_elem, _), ChifType::Array(actual_elem, _)) => {
                self.types_compatible(expected_elem, actual_elem)
            }
            (ChifType::List(expected_elem, _), ChifType::List(actual_elem, _)) => {
                self.types_compatible(expected_elem, actual_elem)
            }
            // Allow array literals to be assigned to list variables
            (ChifType::List(expected_elem, _), ChifType::Array(actual_elem, _)) => {
                self.types_compatible(expected_elem, actual_elem)
            }
            // Allow list literals to be assigned to array variables
            (ChifType::Array(expected_elem, _), ChifType::List(actual_elem, _)) => {
                self.types_compatible(expected_elem, actual_elem)
            }
            
            // Map compatibility
            (ChifType::Map(expected_key, expected_val), ChifType::Map(actual_key, actual_val)) => {
                self.types_compatible(expected_key, actual_key) && 
                self.types_compatible(expected_val, actual_val)
            }
            
            // Struct compatibility
            (ChifType::Struct(expected_name), ChifType::Struct(actual_name)) => {
                expected_name == actual_name
            }
            
            // Pointer compatibility
            (ChifType::Pointer(expected_inner), ChifType::Pointer(actual_inner)) => {
                self.types_compatible(expected_inner, actual_inner)
            }
            
            // Nil can be assigned to any pointer type
            (ChifType::Pointer(_), ChifType::Nil) => true,
            
            _ => false,
        }
    }
    
    fn block_always_returns(&self, block: &Block) -> bool {
        for statement in &block.statements {
            if self.statement_always_returns(statement) {
                return true;
            }
        }
        false
    }
    
    fn statement_always_returns(&self, statement: &Statement) -> bool {
        match statement {
            Statement::Return(_) => true,
            Statement::If(if_stmt) => {
                // If statement returns if both branches return
                if let Some(else_block) = &if_stmt.else_block {
                    self.block_always_returns(&if_stmt.then_block) && 
                    self.block_always_returns(else_block)
                } else {
                    false
                }
            }
            Statement::Switch(switch_stmt) => {
                // Switch returns if all cases return and there's a default case
                if switch_stmt.default_case.is_none() {
                    return false;
                }
                
                // Check all cases return
                for case in &switch_stmt.cases {
                    if !self.block_always_returns(&case.body) {
                        return false;
                    }
                }
                
                // Check default case returns
                if let Some(default_case) = &switch_stmt.default_case {
                    self.block_always_returns(default_case)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    pub fn analyze(&mut self, program: &Program) -> Result<AnalyzedProgram, SemanticError> {
        // First pass: collect all function and struct definitions
        self.collect_definitions(program)?;
        
        // Second pass: analyze function bodies and expressions
        self.analyze_program(program)?;
        
        // Third pass: detailed type checking
        self.check_types(program)?;
        
        Ok(AnalyzedProgram {
            items: program.items.clone(), // TODO: Replace with analyzed items
        })
    }
    
    fn collect_definitions(&mut self, program: &Program) -> Result<(), SemanticError> {
        // Add built-in functions
        self.add_builtin_functions()?;
        
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    let signature = FunctionSignature {
                        name: func.name.clone(),
                        parameters: func.params.clone(),
                        return_type: func.return_type.clone().unwrap_or(ChifType::Nil),
                        is_mutating: false,  // Обычные функции по умолчанию не мутируют
                    };
                    
                    let symbol = Symbol {
                        name: func.name.clone(),
                        symbol_type: SymbolType::Function(signature),
                        location: SourceLocation::unknown(),
                        is_mutable: false,
                    };
                    
                    self.symbol_table.define_symbol(symbol)?;
                }
                Item::Struct(struct_def) => {
                    let struct_definition = StructDefinition {
                        name: struct_def.name.clone(),
                        fields: struct_def.fields.clone(),
                    };
                    
                    let symbol = Symbol {
                        name: struct_def.name.clone(),
                        symbol_type: SymbolType::Struct(struct_definition),
                        location: SourceLocation::unknown(),
                        is_mutable: false,
                    };
                    
                    self.symbol_table.define_symbol(symbol)?;
                }
                Item::StructImpl(impl_block) => {
                    // Add methods to symbol table with struct prefix
                    for method in &impl_block.methods {
                        let method_name = format!("{}_{}", impl_block.struct_name, method.name);
                        
                        // Анализируем тело метода для определения мутабельности
                        let is_mutating = self.analyze_method_mutability(method);
                        
                        let signature = FunctionSignature {
                            name: method_name.clone(),
                            parameters: method.params.clone(),
                            return_type: method.return_type.clone().unwrap_or(ChifType::Nil),
                            is_mutating,  // Устанавливаем флаг мутабельности
                        };
                        
                        let symbol = Symbol {
                            name: method_name,
                            symbol_type: SymbolType::Function(signature),
                            location: SourceLocation::unknown(),
                            is_mutable: false,
                        };
                        
                        self.symbol_table.define_symbol(symbol)?;
                    }
                }
                Item::Import(import) => {
                    // Process imports in the first pass to make symbols available
                    self.process_import(import)?;
                }
                _ => {} // Other items will be handled in the second pass
            }
        }
        
        Ok(())
    }
    
    fn analyze_program(&mut self, program: &Program) -> Result<(), SemanticError> {
        for item in &program.items {
            self.analyze_item(item)?;
        }
        
        Ok(())
    }
    
    fn analyze_item(&mut self, item: &Item) -> Result<(), SemanticError> {
        match item {
            Item::Function(func) => {
                // Create new scope for function
                self.symbol_table.push_scope();
                
                // Set current function return type for validation
                let old_return_type = self.current_function_return_type.clone();
                self.current_function_return_type = func.return_type.clone();
                
                // Add parameters to function scope
                for param in &func.params {
                    // For reference parameters, the type is already a pointer type
                    // We don't need to wrap it again
                    let symbol = Symbol {
                        name: param.name.clone(),
                        symbol_type: SymbolType::Variable(param.param_type.clone()),
                        location: SourceLocation::unknown(),
                        is_mutable: param.is_reference, // Reference parameters are mutable
                    };
                    
                    self.symbol_table.define_symbol(symbol)?;
                }
                
                // Analyze function body
                self.analyze_block(&func.body)?;
                
                // Restore previous function return type
                self.current_function_return_type = old_return_type;
                
                // Pop function scope
                self.symbol_table.pop_scope()?;
            }
            Item::Struct(_) => {
                // Struct definitions are already handled in collect_definitions
            }
            Item::StructImpl(impl_block) => {
                // Analyze methods in struct implementation
                for method in &impl_block.methods {
                    self.analyze_item(&Item::Function(method.clone()))?;
                }
            }
            Item::Import(_) => {
                // Imports are already processed in collect_definitions
            }
        }
        
        Ok(())
    }
    
    fn analyze_block(&mut self, block: &Block) -> Result<(), SemanticError> {
        for statement in &block.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }
    
    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), SemanticError> {
        match statement {
            Statement::VarDecl(var_decl) => {
                // Analyze the initial value if present
                if let Some(expr) = &var_decl.value {
                    let _expr_type = self.analyze_expression(expr)?;
                    // TODO: Check type compatibility
                }
                
                let symbol = Symbol {
                    name: var_decl.name.clone(),
                    symbol_type: SymbolType::Variable(var_decl.var_type.clone()),
                    location: SourceLocation::unknown(),
                    is_mutable: var_decl.is_mutable,
                };
                
                self.symbol_table.define_symbol(symbol)?;
            }
            Statement::Assignment(assignment) => {
                self.analyze_expression(&assignment.target)?;
                self.analyze_expression(&assignment.value)?;
                // TODO: Check assignment compatibility
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.analyze_expression(expr)?;
                }
            }
            Statement::If(if_stmt) => {
                self.analyze_expression(&if_stmt.condition)?;
                self.analyze_block(&if_stmt.then_block)?;
                if let Some(else_block) = &if_stmt.else_block {
                    self.analyze_block(else_block)?;
                }
            }
            Statement::While(while_stmt) => {
                self.analyze_expression(&while_stmt.condition)?;
                
                // Set loop context
                let old_in_loop = self.in_loop;
                self.in_loop = true;
                
                self.analyze_block(&while_stmt.body)?;
                
                // Restore loop context
                self.in_loop = old_in_loop;
            }
            Statement::For(for_stmt) => {
                self.symbol_table.push_scope();
                
                if let Some(init) = &for_stmt.init {
                    self.analyze_statement(init)?;
                }
                if let Some(condition) = &for_stmt.condition {
                    self.analyze_expression(condition)?;
                }
                if let Some(update) = &for_stmt.update {
                    self.analyze_statement(update)?;
                }
                
                // Set loop context
                let old_in_loop = self.in_loop;
                self.in_loop = true;
                
                self.analyze_block(&for_stmt.body)?;
                
                // Restore loop context
                self.in_loop = old_in_loop;
                
                self.symbol_table.pop_scope()?;
            }
            Statement::Switch(switch_stmt) => {
                self.analyze_expression(&switch_stmt.expr)?;
                for case in &switch_stmt.cases {
                    self.analyze_expression(&case.value)?;
                    self.analyze_block(&case.body)?;
                }
                if let Some(default_case) = &switch_stmt.default_case {
                    self.analyze_block(default_case)?;
                }
            }
            Statement::Break => {
                // Check if we're in a loop context
                if !self.in_loop {
                    return Err(SemanticError::InvalidBreak);
                }
            }
            Statement::Continue => {
                // Check if we're in a loop context
                if !self.in_loop {
                    return Err(SemanticError::InvalidContinue);
                }
            }
        }
        
        Ok(())
    }
    
    fn analyze_expression(&mut self, expression: &Expression) -> Result<ChifType, SemanticError> {
        match expression {
            Expression::Literal(value) => {
                Ok(match value {
                    ChifValue::Int(_) => ChifType::Int,
                    ChifValue::Float(_) => ChifType::Float,
                    ChifValue::Str(_) => ChifType::Str,
                    ChifValue::Bool(_) => ChifType::Bool,
                    ChifValue::Nil => ChifType::Nil,
                    ChifValue::Array(_) => ChifType::Array(Box::new(ChifType::Nil), vec![0]), // TODO: Proper array type
                    ChifValue::List(_) => ChifType::List(Box::new(ChifType::Nil), vec![]), // TODO: Proper list type
                    ChifValue::Map(_) => ChifType::Map(Box::new(ChifType::Nil), Box::new(ChifType::Nil)), // TODO: Proper map type
                    ChifValue::Struct(_, _) => ChifType::Nil, // TODO: Proper struct type
                    ChifValue::Pointer(_) => ChifType::Pointer(Box::new(ChifType::Nil)), // TODO: Proper pointer type
                    ChifValue::Reference(_) => ChifType::Pointer(Box::new(ChifType::Nil)), // TODO: Proper reference type
                })
            }
            Expression::Identifier(name) => {
                if let Some(symbol) = self.symbol_table.lookup_symbol(name) {
                    match &symbol.symbol_type {
                        SymbolType::Variable(var_type) => Ok(var_type.clone()),
                        _ => Err(SemanticError::InvalidOperation {
                            location: SourceLocation::unknown(),
                            message: format!("'{}' is not a variable", name),
                        }),
                    }
                } else {
                    Err(SemanticError::UndefinedSymbol {
                        symbol: name.clone(),
                        location: SourceLocation::unknown(),
                    })
                }
            }
            Expression::Binary(binary_op) => {
                let left_type = self.analyze_expression(&binary_op.left)?;
                let right_type = self.analyze_expression(&binary_op.right)?;
                
                match binary_op.operator {
                    BinaryOperator::Add | BinaryOperator::Subtract | 
                    BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => {
                        // Arithmetic operations
                        match (&left_type, &right_type) {
                            (ChifType::Int, ChifType::Int) => Ok(ChifType::Int),
                            (ChifType::Float, ChifType::Float) => Ok(ChifType::Float),
                            (ChifType::Int, ChifType::Float) | (ChifType::Float, ChifType::Int) => Ok(ChifType::Float),
                            (ChifType::Str, ChifType::Str) if binary_op.operator == BinaryOperator::Add => Ok(ChifType::Str),
                            _ => Err(SemanticError::TypeMismatch {
                                location: SourceLocation::unknown(),
                                expected: left_type.clone(),
                                found: right_type,
                            }),
                        }
                    }
                    BinaryOperator::Equal | BinaryOperator::NotEqual => {
                        // Equality operations - can compare any types
                        Ok(ChifType::Bool)
                    }
                    BinaryOperator::Less | BinaryOperator::Greater | 
                    BinaryOperator::LessEqual | BinaryOperator::GreaterEqual => {
                        // Comparison operations
                        match (&left_type, &right_type) {
                            (ChifType::Int, ChifType::Int) | (ChifType::Float, ChifType::Float) |
                            (ChifType::Int, ChifType::Float) | (ChifType::Float, ChifType::Int) |
                            (ChifType::Str, ChifType::Str) => Ok(ChifType::Bool),
                            _ => Err(SemanticError::TypeMismatch {
                                location: SourceLocation::unknown(),
                                expected: left_type.clone(),
                                found: right_type,
                            }),
                        }
                    }
                    BinaryOperator::And | BinaryOperator::Or => {
                        // Logical operations
                        if left_type == ChifType::Bool && right_type == ChifType::Bool {
                            Ok(ChifType::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                location: SourceLocation::unknown(),
                                expected: ChifType::Bool,
                                found: if left_type != ChifType::Bool { left_type } else { right_type },
                            })
                        }
                    }
                }
            }
            Expression::Unary(unary_op) => {
                let operand_type = self.analyze_expression(&unary_op.operand)?;
                
                match unary_op.operator {
                    UnaryOperator::Minus => {
                        match operand_type {
                            ChifType::Int => Ok(ChifType::Int),
                            ChifType::Float => Ok(ChifType::Float),
                            _ => Err(SemanticError::InvalidOperation {
                                location: SourceLocation::unknown(),
                                message: format!("Cannot apply unary minus to type {:?}", operand_type),
                            }),
                        }
                    }
                    UnaryOperator::Not => {
                        if operand_type == ChifType::Bool {
                            Ok(ChifType::Bool)
                        } else {
                            Err(SemanticError::TypeMismatch {
                                location: SourceLocation::unknown(),
                                expected: ChifType::Bool,
                                found: operand_type,
                            })
                        }
                    }
                }
            }
            Expression::Call(func_call) => {
                // Analyze arguments first
                let mut arg_types = Vec::new();
                for arg in &func_call.args {
                    arg_types.push(self.analyze_expression(arg)?);
                }
                
                // Check if function exists
                if let Some(symbol) = self.symbol_table.lookup_symbol(&func_call.name) {
                    match &symbol.symbol_type {
                        SymbolType::Function(signature) => {
                            // Check argument count
                            if arg_types.len() != signature.parameters.len() {
                                return Err(SemanticError::InvalidOperation {
                                    location: SourceLocation::unknown(),
                                    message: format!(
                                        "Function '{}' expects {} arguments, got {}",
                                        func_call.name,
                                        signature.parameters.len(),
                                        arg_types.len()
                                    ),
                                });
                            }
                            
                            // Check argument types
                            for (_i, (arg_type, param)) in arg_types.iter().zip(&signature.parameters).enumerate() {
                                if param.is_reference {
                                    // For reference parameters, the argument should match the parameter type
                                    // (which is already a pointer type)
                                    if !self.types_compatible(&param.param_type, arg_type) {
                                        return Err(SemanticError::TypeMismatch {
                                            location: SourceLocation::unknown(),
                                            expected: param.param_type.clone(),
                                            found: arg_type.clone(),
                                        });
                                    }
                                } else {
                                    // For value parameters, check type compatibility directly
                                    if !self.types_compatible(&param.param_type, arg_type) {
                                        return Err(SemanticError::TypeMismatch {
                                            location: SourceLocation::unknown(),
                                            expected: param.param_type.clone(),
                                            found: arg_type.clone(),
                                        });
                                    }
                                }
                            }
                            
                            Ok(signature.return_type.clone())
                        }
                        _ => Err(SemanticError::InvalidOperation {
                            location: SourceLocation::unknown(),
                            message: format!("'{}' is not a function", func_call.name),
                        }),
                    }
                } else {
                    Err(SemanticError::UndefinedSymbol {
                        symbol: func_call.name.clone(),
                        location: SourceLocation::unknown(),
                    })
                }
            }
            Expression::StructLiteral(struct_literal) => {
                // Check if struct exists
                if let Some(symbol) = self.symbol_table.lookup_symbol(&struct_literal.struct_name) {
                    match &symbol.symbol_type {
                        SymbolType::Struct(struct_def) => {
                            let struct_def = struct_def.clone(); // Clone to avoid borrow issues
                            
                            // Check that all required fields are provided
                            for field in &struct_def.fields {
                                let field_provided = struct_literal.fields.iter()
                                    .any(|(name, _)| name == &field.name);
                                if !field_provided {
                                    return Err(SemanticError::InvalidOperation {
                                        location: SourceLocation::unknown(),
                                        message: format!(
                                            "Missing field '{}' in struct literal for '{}'",
                                            field.name, struct_literal.struct_name
                                        ),
                                    });
                                }
                            }
                            
                            // Check field types
                            for (field_name, field_expr) in &struct_literal.fields {
                                let expr_type = self.analyze_expression(field_expr)?;
                                
                                // Find the field definition
                                if let Some(field_def) = struct_def.fields.iter()
                                    .find(|f| f.name == *field_name) {
                                    if !self.types_compatible(&field_def.field_type, &expr_type) {
                                        return Err(SemanticError::TypeMismatch {
                                            location: SourceLocation::unknown(),
                                            expected: field_def.field_type.clone(),
                                            found: expr_type,
                                        });
                                    }
                                } else {
                                    return Err(SemanticError::InvalidOperation {
                                        location: SourceLocation::unknown(),
                                        message: format!(
                                            "Unknown field '{}' in struct '{}'",
                                            field_name, struct_literal.struct_name
                                        ),
                                    });
                                }
                            }
                            
                            Ok(ChifType::Struct(struct_literal.struct_name.clone()))
                        }
                        _ => Err(SemanticError::InvalidOperation {
                            location: SourceLocation::unknown(),
                            message: format!("'{}' is not a struct", struct_literal.struct_name),
                        }),
                    }
                } else {
                    Err(SemanticError::UndefinedSymbol {
                        symbol: struct_literal.struct_name.clone(),
                        location: SourceLocation::unknown(),
                    })
                }
            }
            Expression::FieldAccess(field_access) => {
                // Analyze the object expression to get its type
                let object_type = self.analyze_expression(&field_access.object)?;
                
                match object_type {
                    ChifType::Struct(struct_name) => {
                        // Look up the struct definition
                        if let Some(symbol) = self.symbol_table.lookup_symbol(&struct_name) {
                            match &symbol.symbol_type {
                                SymbolType::Struct(struct_def) => {
                                    // Find the field in the struct definition
                                    if let Some(field) = struct_def.fields.iter()
                                        .find(|f| f.name == field_access.field) {
                                        Ok(field.field_type.clone())
                                    } else {
                                        Err(SemanticError::InvalidOperation {
                                            location: SourceLocation::unknown(),
                                            message: format!(
                                                "Field '{}' not found in struct '{}'",
                                                field_access.field, struct_name
                                            ),
                                        })
                                    }
                                }
                                _ => Err(SemanticError::InvalidOperation {
                                    location: SourceLocation::unknown(),
                                    message: format!("'{}' is not a struct", struct_name),
                                }),
                            }
                        } else {
                            Err(SemanticError::UndefinedSymbol {
                                symbol: struct_name,
                                location: SourceLocation::unknown(),
                            })
                        }
                    }
                    _ => Err(SemanticError::InvalidOperation {
                        location: SourceLocation::unknown(),
                        message: format!("Cannot access field '{}' on non-struct type {:?}", field_access.field, object_type),
                    }),
                }
            }
            Expression::MethodCall(method_call) => {
                // Special handling for console I/O
                if let Expression::Identifier(object_name) = &*method_call.object {
                    if object_name == "con" && method_call.method == "out" {
                        // Analyze arguments for con.out
                        for arg in &method_call.args {
                            self.analyze_expression(arg)?;
                        }
                        return Ok(ChifType::Nil); // con.out returns void
                    } else if object_name == "con" && method_call.method == "in" {
                        // con.in takes no arguments and returns int for now
                        if !method_call.args.is_empty() {
                            return Err(SemanticError::InvalidOperation {
                                location: SourceLocation::unknown(),
                                message: "con.in expects no arguments".to_string(),
                            });
                        }
                        return Ok(ChifType::Int); // con.in returns int for now

                    } else if object_name == "http" && method_call.method == "get" {
                        // http.get(url) returns string
                        if method_call.args.len() != 1 {
                            return Err(SemanticError::InvalidOperation {
                                location: SourceLocation::unknown(),
                                message: "http.get expects 1 argument (url)".to_string(),
                            });
                        }
                        // Analyze argument
                        let arg_type = self.analyze_expression(&method_call.args[0])?;
                        if arg_type != ChifType::Str {
                            return Err(SemanticError::TypeMismatch {
                                location: SourceLocation::unknown(),
                                expected: ChifType::Str,
                                found: arg_type,
                            });
                        }
                        return Ok(ChifType::Str);
                    } else if object_name == "http" && method_call.method == "post" {
                        // http.post(url, data) returns string
                        if method_call.args.len() != 2 {
                            return Err(SemanticError::InvalidOperation {
                                location: SourceLocation::unknown(),
                                message: "http.post expects 2 arguments (url, data)".to_string(),
                            });
                        }
                        // Analyze arguments
                        for arg in &method_call.args {
                            let arg_type = self.analyze_expression(arg)?;
                            if arg_type != ChifType::Str {
                                return Err(SemanticError::TypeMismatch {
                                    location: SourceLocation::unknown(),
                                    expected: ChifType::Str,
                                    found: arg_type,
                                });
                            }
                        }
                        return Ok(ChifType::Str);
                    } else if object_name == "http" && method_call.method == "put" {
                        // http.put(url, data) returns string
                        if method_call.args.len() != 2 {
                            return Err(SemanticError::InvalidOperation {
                                location: SourceLocation::unknown(),
                                message: "http.put expects 2 arguments (url, data)".to_string(),
                            });
                        }
                        // Analyze arguments
                        for arg in &method_call.args {
                            let arg_type = self.analyze_expression(arg)?;
                            if arg_type != ChifType::Str {
                                return Err(SemanticError::TypeMismatch {
                                    location: SourceLocation::unknown(),
                                    expected: ChifType::Str,
                                    found: arg_type,
                                });
                            }
                        }
                        return Ok(ChifType::Str);
                    } else if object_name == "http" && method_call.method == "delete" {
                        // http.delete(url) returns string
                        if method_call.args.len() != 1 {
                            return Err(SemanticError::InvalidOperation {
                                location: SourceLocation::unknown(),
                                message: "http.delete expects 1 argument (url)".to_string(),
                            });
                        }
                        // Analyze argument
                        let arg_type = self.analyze_expression(&method_call.args[0])?;
                        if arg_type != ChifType::Str {
                            return Err(SemanticError::TypeMismatch {
                                location: SourceLocation::unknown(),
                                expected: ChifType::Str,
                                found: arg_type,
                            });
                        }
                        return Ok(ChifType::Str);
                    }
                }
                
                // Analyze the object expression to get its type
                let object_type = self.analyze_expression(&method_call.object)?;
                
                // Analyze arguments
                let mut arg_types = Vec::new();
                for arg in &method_call.args {
                    arg_types.push(self.analyze_expression(arg)?);
                }
                
                match object_type {
                    ChifType::Struct(struct_name) => {
                        // Look for method in struct implementation
                        // For now, we'll construct the method name as struct_name + "_" + method_name
                        let method_name = format!("{}_{}", struct_name, method_call.method);
                        
                        if let Some(symbol) = self.symbol_table.lookup_symbol(&method_name) {
                            match &symbol.symbol_type {
                                SymbolType::Function(signature) => {
                                    // Check argument count (excluding self parameter)
                                    let expected_args = signature.parameters.len().saturating_sub(1); // Subtract self parameter
                                    if arg_types.len() != expected_args {
                                        return Err(SemanticError::InvalidOperation {
                                            location: SourceLocation::unknown(),
                                            message: format!(
                                                "Method '{}' expects {} arguments, got {}",
                                                method_call.method,
                                                expected_args,
                                                arg_types.len()
                                            ),
                                        });
                                    }
                                    
                                    // Check argument types (skip first parameter which is self)
                                    for (_i, (arg_type, param)) in arg_types.iter().zip(signature.parameters.iter().skip(1)).enumerate() {
                                        if !self.types_compatible(&param.param_type, arg_type) {
                                            return Err(SemanticError::TypeMismatch {
                                                location: SourceLocation::unknown(),
                                                expected: param.param_type.clone(),
                                                found: arg_type.clone(),
                                            });
                                        }
                                    }
                                    
                                    Ok(signature.return_type.clone())
                                }
                                _ => Err(SemanticError::InvalidOperation {
                                    location: SourceLocation::unknown(),
                                    message: format!("'{}' is not a method", method_name),
                                }),
                            }
                        } else {
                            Err(SemanticError::UndefinedSymbol {
                                symbol: method_name,
                                location: SourceLocation::unknown(),
                            })
                        }
                    }
                    _ => Err(SemanticError::InvalidOperation {
                        location: SourceLocation::unknown(),
                        message: format!("Cannot call method '{}' on non-struct type {:?}", method_call.method, object_type),
                    }),
                }
            }
            Expression::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    // Empty array - we need type inference or explicit type annotation
                    return Ok(ChifType::Array(Box::new(ChifType::Nil), vec![0]));
                }
                
                // Analyze first element to determine array type
                let first_type = self.analyze_expression(&elements[0])?;
                
                // Check that all elements have the same type
                for (_i, element) in elements.iter().enumerate().skip(1) {
                    let element_type = self.analyze_expression(element)?;
                    if !self.types_compatible(&first_type, &element_type) {
                        return Err(SemanticError::TypeMismatch {
                            location: SourceLocation::unknown(),
                            expected: first_type.clone(),
                            found: element_type,
                        });
                    }
                }
                
                // For multidimensional arrays, we need to handle nested dimensions
                match &first_type {
                    ChifType::Array(inner_type, inner_dims) => {
                        // This is a multidimensional array
                        let mut new_dims = vec![elements.len()];
                        new_dims.extend(inner_dims);
                        Ok(ChifType::Array(inner_type.clone(), new_dims))
                    }
                    _ => {
                        // This is a single-dimensional array
                        Ok(ChifType::Array(Box::new(first_type), vec![elements.len()]))
                    }
                }
            }
            Expression::Index(index_access) => {
                // Analyze the array expression
                let array_type = self.analyze_expression(&index_access.object)?;
                
                // Analyze all index expressions
                for index_expr in &index_access.indices {
                    let index_type = self.analyze_expression(index_expr)?;
                    
                    // Check that index is an integer
                    if index_type != ChifType::Int {
                        return Err(SemanticError::TypeMismatch {
                            location: SourceLocation::unknown(),
                            expected: ChifType::Int,
                            found: index_type,
                        });
                    }
                }
                
                // Check that object is an array and return element type
                match array_type {
                    ChifType::Array(element_type, dimensions) => {
                        // For now, assume single-dimensional indexing
                        if index_access.indices.len() == 1 {
                            Ok(*element_type)
                        } else {
                            // Multi-dimensional indexing - return array of lower dimension
                            let remaining_dims: Vec<usize> = dimensions.into_iter().skip(index_access.indices.len()).collect();
                            if remaining_dims.is_empty() {
                                Ok(*element_type)
                            } else {
                                Ok(ChifType::Array(element_type, remaining_dims))
                            }
                        }
                    }
                    ChifType::List(element_type, _) => Ok(*element_type),
                    _ => Err(SemanticError::InvalidOperation {
                        location: SourceLocation::unknown(),
                        message: format!("Cannot index non-array type {:?}", array_type),
                    }),
                }
            }
            Expression::Reference(expr) => {
                // Address-of operation (&expr) returns a pointer to the expression's type
                let expr_type = self.analyze_expression(expr)?;
                Ok(ChifType::Pointer(Box::new(expr_type)))
            }
            Expression::Dereference(expr) => {
                // Dereference operation (*expr) returns the type pointed to by the expression
                let expr_type = self.analyze_expression(expr)?;
                match expr_type {
                    ChifType::Pointer(inner_type) => Ok(*inner_type),
                    _ => Err(SemanticError::InvalidOperation {
                        location: SourceLocation::unknown(),
                        message: format!("Cannot dereference non-pointer type {:?}", expr_type),
                    }),
                }
            }
            _ => {
                // TODO: Handle other expression types
                Ok(ChifType::Nil)
            }
        }
    }
    
    fn add_builtin_functions(&mut self) -> Result<(), SemanticError> {
        // Add console object 'con'
        let con_symbol = Symbol {
            name: "con".to_string(),
            symbol_type: SymbolType::Variable(ChifType::Struct("Console".to_string())),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        
        self.symbol_table.define_symbol(con_symbol)?;
        
        // Add random functions as global functions
        let randi_signature = FunctionSignature {
            name: "randi".to_string(),
            parameters: vec![
                Parameter { name: "min".to_string(), param_type: ChifType::Int, is_reference: false },
                Parameter { name: "max".to_string(), param_type: ChifType::Int, is_reference: false },
            ],
            return_type: ChifType::Int,
            is_mutating: false,  // Встроенные функции не мутируют
        };
        let randi_symbol = Symbol {
            name: "randi".to_string(),
            symbol_type: SymbolType::Function(randi_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(randi_symbol)?;
        
        let randf_signature = FunctionSignature {
            name: "randf".to_string(),
            parameters: vec![
                Parameter { name: "min".to_string(), param_type: ChifType::Float, is_reference: false },
                Parameter { name: "max".to_string(), param_type: ChifType::Float, is_reference: false },
            ],
            return_type: ChifType::Float,
            is_mutating: false,  // Встроенные функции не мутируют
        };
        let randf_symbol = Symbol {
            name: "randf".to_string(),
            symbol_type: SymbolType::Function(randf_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(randf_symbol)?;
        
        let rands_signature = FunctionSignature {
            name: "rands".to_string(),
            parameters: vec![
                Parameter { name: "from".to_string(), param_type: ChifType::Str, is_reference: false },
                Parameter { name: "to".to_string(), param_type: ChifType::Str, is_reference: false },
            ],
            return_type: ChifType::Str,
            is_mutating: false,  // Встроенные функции не мутируют
        };
        let rands_symbol = Symbol {
            name: "rands".to_string(),
            symbol_type: SymbolType::Function(rands_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(rands_symbol)?;
        
        // Добавляем функции конвертации типов
        // int() может принимать строку или число с плавающей точкой
        let int_signature = FunctionSignature {
            name: "int".to_string(),
            parameters: vec![
                Parameter { name: "value".to_string(), param_type: ChifType::Float, is_reference: false },
            ],
            return_type: ChifType::Int,
            is_mutating: false,
        };
        let int_symbol = Symbol {
            name: "int".to_string(),
            symbol_type: SymbolType::Function(int_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(int_symbol)?;
        
        let int_str_signature = FunctionSignature {
            name: "int".to_string(),
            parameters: vec![
                Parameter { name: "value".to_string(), param_type: ChifType::Str, is_reference: false },
            ],
            return_type: ChifType::Int,
            is_mutating: false,
        };
        let int_str_symbol = Symbol {
            name: "int".to_string(),
            symbol_type: SymbolType::Function(int_str_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(int_str_symbol)?;
        
        // float() может принимать строку или целое число
        let float_signature = FunctionSignature {
            name: "float".to_string(),
            parameters: vec![
                Parameter { name: "value".to_string(), param_type: ChifType::Int, is_reference: false },
            ],
            return_type: ChifType::Float,
            is_mutating: false,
        };
        let float_symbol = Symbol {
            name: "float".to_string(),
            symbol_type: SymbolType::Function(float_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(float_symbol)?;
        
        let float_str_signature = FunctionSignature {
            name: "float".to_string(),
            parameters: vec![
                Parameter { name: "value".to_string(), param_type: ChifType::Str, is_reference: false },
            ],
            return_type: ChifType::Float,
            is_mutating: false,
        };
        let float_str_symbol = Symbol {
            name: "float".to_string(),
            symbol_type: SymbolType::Function(float_str_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(float_str_symbol)?;
        
        // str() может принимать целое число или число с плавающей точкой
        let str_int_signature = FunctionSignature {
            name: "str".to_string(),
            parameters: vec![
                Parameter { name: "value".to_string(), param_type: ChifType::Int, is_reference: false },
            ],
            return_type: ChifType::Str,
            is_mutating: false,
        };
        let str_int_symbol = Symbol {
            name: "str".to_string(),
            symbol_type: SymbolType::Function(str_int_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(str_int_symbol)?;
        
        let str_float_signature = FunctionSignature {
            name: "str".to_string(),
            parameters: vec![
                Parameter { name: "value".to_string(), param_type: ChifType::Float, is_reference: false },
            ],
            return_type: ChifType::Str,
            is_mutating: false,
        };
        let str_float_symbol = Symbol {
            name: "str".to_string(),
            symbol_type: SymbolType::Function(str_float_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        let float_signature = FunctionSignature {
            name: "float".to_string(),
            parameters: vec![
                Parameter { name: "value".to_string(), param_type: ChifType::Str, is_reference: false },
            ],
            return_type: ChifType::Float,
            is_mutating: false,
        };
        let float_symbol = Symbol {
            name: "float".to_string(),
            symbol_type: SymbolType::Function(float_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(float_symbol)?;
        
        // str() может принимать любой тип, но мы укажем Int для семантического анализатора
        let str_signature = FunctionSignature {
            name: "str".to_string(),
            parameters: vec![
                Parameter { name: "value".to_string(), param_type: ChifType::Int, is_reference: false },
            ],
            return_type: ChifType::Str,
            is_mutating: false,
        };
        let str_symbol = Symbol {
            name: "str".to_string(),
            symbol_type: SymbolType::Function(str_signature),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        self.symbol_table.define_symbol(str_symbol)?;
        
        // Add HTTP object 'http'
        let http_symbol = Symbol {
            name: "http".to_string(),
            symbol_type: SymbolType::Variable(ChifType::Struct("Http".to_string())),
            location: SourceLocation::unknown(),
            is_mutable: false,
        };
        
        self.symbol_table.define_symbol(http_symbol)?;
        
        Ok(())
    }
    
    fn process_import(&mut self, import: &ImportStatement) -> Result<(), SemanticError> {
        // Add .rono extension if not present
        let file_path = if import.path.ends_with(".rono") {
            import.path.clone()
        } else {
            format!("{}.rono", import.path)
        };
        
        // Read the imported file
        let source = fs::read_to_string(&file_path).map_err(|_| {
            SemanticError::InvalidOperation {
                location: SourceLocation::unknown(),
                message: format!("Could not read module file: {}", file_path),
            }
        })?;
        
        // Parse the imported file
        use crate::{lexer::Lexer, parser::Parser};
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize().map_err(|e| {
            SemanticError::InvalidOperation {
                location: SourceLocation::unknown(),
                message: format!("Failed to tokenize module {}: {}", file_path, e),
            }
        })?;
        
        let mut parser = Parser::new(tokens);
        let imported_program = parser.parse().map_err(|e| {
            SemanticError::InvalidOperation {
                location: SourceLocation::unknown(),
                message: format!("Failed to parse module {}: {}", file_path, e),
            }
        })?;
        
        // Extract functions and structs from imported module
        let mut module_functions = HashMap::new();
        let mut module_structs = HashMap::new();
        
        for item in &imported_program.items {
            match item {
                Item::Function(func) => {
                    let signature = FunctionSignature {
                        name: func.name.clone(),
                        parameters: func.params.clone(),
                        return_type: func.return_type.clone().unwrap_or(ChifType::Nil),
                        is_mutating: false,  // Импортированные функции по умолчанию не мутируют
                    };
                    module_functions.insert(func.name.clone(), signature.clone());
                    
                    // Add function to global symbol table with module prefix
                    let module_name = import.alias.clone().unwrap_or_else(|| {
                        std::path::Path::new(&import.path)
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_string()
                    });
                    
                    let qualified_name = format!("{}_{}", module_name, func.name);
                    let symbol = Symbol {
                        name: qualified_name,
                        symbol_type: SymbolType::Function(signature),
                        location: SourceLocation::unknown(),
                        is_mutable: false,
                    };
                    
                    self.symbol_table.define_symbol(symbol)?;
                }
                Item::Struct(struct_def) => {
                    let struct_definition = StructDefinition {
                        name: struct_def.name.clone(),
                        fields: struct_def.fields.clone(),
                    };
                    module_structs.insert(struct_def.name.clone(), struct_definition.clone());
                    
                    // Add struct to global symbol table with module prefix
                    let module_name = import.alias.clone().unwrap_or_else(|| {
                        std::path::Path::new(&import.path)
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_string()
                    });
                    
                    let qualified_name = format!("{}_{}", module_name, struct_def.name);
                    let symbol = Symbol {
                        name: qualified_name,
                        symbol_type: SymbolType::Struct(struct_definition),
                        location: SourceLocation::unknown(),
                        is_mutable: false,
                    };
                    
                    self.symbol_table.define_symbol(symbol)?;
                }
                Item::StructImpl(impl_block) => {
                    // Add methods to symbol table with module and struct prefix
                    let module_name = import.alias.clone().unwrap_or_else(|| {
                        std::path::Path::new(&import.path)
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_string()
                    });
                    
                    for method in &impl_block.methods {
                        let method_name = format!("{}_{}_{}", module_name, impl_block.struct_name, method.name);
                        let signature = FunctionSignature {
                            name: method_name.clone(),
                            parameters: method.params.clone(),
                            return_type: method.return_type.clone().unwrap_or(ChifType::Nil),
                            is_mutating: false,  // Методы импортированных структур по умолчанию не мутируют
                        };
                        
                        let symbol = Symbol {
                            name: method_name,
                            symbol_type: SymbolType::Function(signature),
                            location: SourceLocation::unknown(),
                            is_mutable: false,
                        };
                        
                        self.symbol_table.define_symbol(symbol)?;
                    }
                }
                _ => {} // Ignore nested imports for now
            }
        }
        
        // Store module information
        let module_name = import.alias.clone().unwrap_or_else(|| {
            std::path::Path::new(&import.path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string()
        });
        
        let module_info = ModuleInfo {
            name: module_name.clone(),
            functions: module_functions,
            structs: module_structs,
        };
        
        self.modules.insert(module_name, module_info);
        
        Ok(())
    }
    
    /// Анализирует тело метода для определения, изменяет ли он поля структуры через self
    fn analyze_method_mutability(&self, method: &Function) -> bool {
        // Проверяем, есть ли параметр self
        let has_self = method.params.iter().any(|param| param.name == "self");
        if !has_self {
            return false; // Если нет self, метод не может быть мутирующим
        }
        
        // Анализируем тело метода на предмет изменения полей self
        self.analyze_block_for_self_mutation(&method.body)
    }
    
    /// Рекурсивно анализирует блок кода на предмет мутации полей self
    fn analyze_block_for_self_mutation(&self, block: &Block) -> bool {
        for statement in &block.statements {
            if self.analyze_statement_for_self_mutation(statement) {
                return true;
            }
        }
        false
    }
    
    /// Анализирует отдельное утверждение на предмет мутации полей self
    fn analyze_statement_for_self_mutation(&self, statement: &Statement) -> bool {
        match statement {
            Statement::Assignment(assignment) => {
                // Проверяем, является ли цель присваивания полем self
                self.is_self_field_access(&assignment.target)
            }
            Statement::If(if_stmt) => {
                // Проверяем оба блока if-else
                let then_mutates = self.analyze_block_for_self_mutation(&if_stmt.then_block);
                let else_mutates = if let Some(else_block) = &if_stmt.else_block {
                    self.analyze_block_for_self_mutation(else_block)
                } else {
                    false
                };
                then_mutates || else_mutates
            }
            Statement::For(for_stmt) => {
                // Проверяем тело цикла
                self.analyze_block_for_self_mutation(&for_stmt.body)
            }
            Statement::While(while_stmt) => {
                // Проверяем тело цикла
                self.analyze_block_for_self_mutation(&while_stmt.body)
            }
            Statement::Switch(switch_stmt) => {
                // Проверяем все случаи switch
                for case in &switch_stmt.cases {
                    if self.analyze_block_for_self_mutation(&case.body) {
                        return true;
                    }
                }
                if let Some(default_case) = &switch_stmt.default_case {
                    return self.analyze_block_for_self_mutation(default_case);
                }
                false
            }
            _ => false, // Другие типы утверждений не мутируют self
        }
    }
    
    /// Проверяет, является ли выражение доступом к полю self (например, self.x)
    fn is_self_field_access(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FieldAccess(field_access) => {
                // Проверяем, является ли объект доступа к полю идентификатором "self"
                match field_access.object.as_ref() {
                    Expression::Identifier(name) => name == "self",
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

// Placeholder for analyzed program
#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    pub items: Vec<Item>,
}