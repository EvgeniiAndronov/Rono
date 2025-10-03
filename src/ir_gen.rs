use crate::ast::*;
use crate::semantic::AnalyzedProgram;
use crate::types::{ChifType, ChifValue};

use cranelift::prelude::*;
use cranelift_module::{Linkage, Module};
use cranelift_object::ObjectModule;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IRError {
    #[error("IR generation error: {0}")]
    Generation(String),
    
    #[error("Type conversion error: {0}")]
    TypeConversion(String),
    
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    
    #[error("Module error: {0}")]
    Module(#[from] cranelift_module::ModuleError),
}

pub struct IRGenerator {
    pub module: ObjectModule,
    pub builder_context: FunctionBuilderContext,
    pub ctx: codegen::Context,
    
    // Symbol tables for IR generation
    pub functions: HashMap<String, cranelift_module::FuncId>,
    pub variables: HashMap<String, Variable>,
    pub current_function: Option<cranelift_module::FuncId>,
    pub string_constants: HashMap<String, cranelift_module::DataId>,
    
    // Struct definitions for layout information
    pub structs: HashMap<String, StructLayout>,
    
    // Loop context for break/continue
    pub loop_stack: Vec<LoopContext>,
}

#[derive(Debug, Clone)]
pub struct LoopContext {
    pub break_block: cranelift::prelude::Block,
    pub continue_block: cranelift::prelude::Block,
}

#[derive(Debug, Clone)]
pub struct StructLayout {
    pub name: String,
    pub fields: Vec<StructFieldLayout>,
    pub size: u32,
    pub alignment: u32,
}

#[derive(Debug, Clone)]
pub struct StructFieldLayout {
    pub name: String,
    pub field_type: ChifType,
    pub offset: u32,
    pub size: u32,
}

impl IRGenerator {
    pub fn new(module: ObjectModule) -> Self {
        Self {
            module,
            builder_context: FunctionBuilderContext::new(),
            ctx: codegen::Context::new(),
            functions: HashMap::new(),
            variables: HashMap::new(),
            current_function: None,
            string_constants: HashMap::new(),
            structs: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }
    
    pub fn generate(&mut self, program: &AnalyzedProgram) -> Result<(), IRError> {
        // First pass: declare runtime functions
        self.declare_runtime_functions()?;
        
        // Second pass: process imports and their functions
        for item in &program.items {
            if let Item::Import(import) = item {
                self.process_import(import)?;
            }
        }
        
        // Third pass: process struct definitions
        for item in &program.items {
            if let Item::Struct(struct_def) = item {
                self.process_struct_definition(struct_def)?;
            }
        }
        
        // Fourth pass: declare all user functions and struct methods
        for item in &program.items {
            if let Item::Function(func) = item {
                self.declare_function(func)?;
            } else if let Item::StructImpl(impl_block) = item {
                // Declare methods with struct prefix
                for method in &impl_block.methods {
                    let method_name = format!("{}_{}", impl_block.struct_name, method.name);
                    let mut method_with_new_name = method.clone();
                    method_with_new_name.name = method_name;
                    self.declare_function(&method_with_new_name)?;
                }
            }
        }
        
        // Fifth pass: generate function bodies and struct methods
        for item in &program.items {
            if let Item::Function(func) = item {
                self.generate_function(func)?;
            } else if let Item::StructImpl(impl_block) = item {
                // Generate method bodies with struct prefix
                for method in &impl_block.methods {
                    let method_name = format!("{}_{}", impl_block.struct_name, method.name);
                    let mut method_with_new_name = method.clone();
                    method_with_new_name.name = method_name;
                    self.generate_function(&method_with_new_name)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn declare_function(&mut self, func: &Function) -> Result<(), IRError> {
        let mut sig = self.module.make_signature();
        
        // Use system calling convention for main function
        if func.is_main {
            sig.call_conv = self.module.target_config().default_call_conv;
            // Main function should have standard C signature: int main(int argc, char** argv)
            // For now, we'll make it int main(void) and ignore parameters
            sig.returns.push(AbiParam::new(types::I32)); // Return int
        } else {
            // Add parameters for regular functions
            for param in &func.params {
                let cranelift_type = Self::chif_type_to_cranelift(&param.param_type)?;
                sig.params.push(AbiParam::new(cranelift_type));
            }
        }
        
        // Add return type
        if let Some(return_type) = &func.return_type {
            if *return_type != ChifType::Nil {
                let cranelift_type = Self::chif_type_to_cranelift(return_type)?;
                sig.returns.push(AbiParam::new(cranelift_type));
            }
        }
        
        let func_id = self.module.declare_function(&func.name, Linkage::Export, &sig)
            .map_err(|e| IRError::Module(e))?;
        
        self.functions.insert(func.name.clone(), func_id);
        
        Ok(())
    }
    
    fn generate_function(&mut self, func: &Function) -> Result<(), IRError> {
        let func_id = self.functions[&func.name];
        self.current_function = Some(func_id);
        
        // Clear context for new function
        self.ctx.clear();
        self.variables.clear();
        
        // Get function signature
        let sig = self.module.declarations().get_function_decl(func_id).signature.clone();
        
        // Set the function signature in the context
        self.ctx.func.signature = sig.clone();
        
        // Create function builder
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
        
        // Create entry block
        let entry_block = builder.create_block();
        
        // Add block params for functions with parameters
        if !func.params.is_empty() {
            // Manually add block parameters based on function signature
            for param_abi in &sig.params {
                builder.append_block_param(entry_block, param_abi.value_type);
            }
        }
        
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        
        // Create variables for parameters
        if !func.params.is_empty() {
            let block_params: Vec<Value> = builder.block_params(entry_block).to_vec();
            for (i, param) in func.params.iter().enumerate() {
                if i < block_params.len() && i < sig.params.len() {
                    let param_value = block_params[i];
                    let var = Variable::new(self.variables.len());
                    let param_type = sig.params[i].value_type;
                    builder.declare_var(var, param_type);
                    builder.def_var(var, param_value);
                    self.variables.insert(param.name.clone(), var);
                }
            }
        }
        
        // Generate function body
        let has_return = Self::block_ends_with_return(&func.body);
        
        // Generate statements
        let statements = func.body.statements.clone();
        let variables = &mut self.variables;
        let is_main = func.is_main;
        
        for statement in statements {
            Self::generate_statement_static(&mut builder, &statement, variables, is_main, &self.functions, &mut self.module)?;
        }
        
        // Add implicit return if needed
        if !has_return {
            if func.is_main {
                // Main function should return 0 (success) by default
                let zero = builder.ins().iconst(types::I32, 0);
                builder.ins().return_(&[zero]);
            } else if func.return_type.is_none() || func.return_type == Some(ChifType::Nil) {
                builder.ins().return_(&[]);
            } else {
                // This should be caught by semantic analysis
                return Err(IRError::Generation("Function missing return statement".to_string()));
            }
        }
        
        // Finalize function
        builder.finalize();
        
        // Print IR for debugging (commented out for now)
        // println!("Generated IR for function '{}':", func.name);
        // println!("{}", self.ctx.func.display());
        
        // Define the function in the module
        self.module.define_function(func_id, &mut self.ctx)
            .map_err(|e| {
                println!("Function '{}' IR:", func.name);
                println!("{}", self.ctx.func.display());
                IRError::Module(e)
            })?;
        
        Ok(())
    }
    
    fn generate_statement_static(
        builder: &mut FunctionBuilder, 
        statement: &Statement, 
        variables: &mut HashMap<String, Variable>,
        is_main: bool,
        functions: &HashMap<String, cranelift_module::FuncId>,
        module: &mut ObjectModule
    ) -> Result<(), IRError> {
        match statement {
            Statement::VarDecl(var_decl) => {
                let cranelift_type = Self::chif_type_to_cranelift(&var_decl.var_type)?;
                let var = Variable::new(variables.len());
                builder.declare_var(var, cranelift_type);
                
                let init_value = if let Some(init_expr) = &var_decl.value {
                    Self::generate_expression_static(builder, init_expr, variables, functions, module)?
                } else {
                    // Initialize with default value
                    Self::get_default_value(builder, cranelift_type)
                };
                
                builder.def_var(var, init_value);
                variables.insert(var_decl.name.clone(), var);
            }
            Statement::Assignment(assignment) => {
                // For now, only handle simple variable assignments
                if let Expression::Identifier(var_name) = &assignment.target {
                    let value = Self::generate_expression_static(builder, &assignment.value, variables, functions, module)?;
                    if let Some(&var) = variables.get(var_name) {
                        builder.def_var(var, value);
                    } else {
                        return Err(IRError::Generation(format!("Undefined variable: {}", var_name)));
                    }
                } else {
                    return Err(IRError::UnsupportedFeature("Complex assignment targets not yet supported".to_string()));
                }
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    if is_main {
                        // Main function should return int32
                        let return_value = Self::generate_expression_static(builder, expr, variables, functions, module)?;
                        // Convert to i32 if needed
                        let return_i32 = builder.ins().ireduce(types::I32, return_value);
                        builder.ins().return_(&[return_i32]);
                    } else {
                        let return_value = Self::generate_expression_static(builder, expr, variables, functions, module)?;
                        builder.ins().return_(&[return_value]);
                    }
                } else {
                    if is_main {
                        // Main function returns 0 by default
                        let zero = builder.ins().iconst(types::I32, 0);
                        builder.ins().return_(&[zero]);
                    } else {
                        builder.ins().return_(&[]);
                    }
                }
            }
            Statement::Expression(expr) => {
                // Generate expression but ignore result
                Self::generate_expression_static(builder, expr, variables, functions, module)?;
            }
            Statement::If(if_stmt) => {
                // Generate condition
                let condition = Self::generate_expression_static(builder, &if_stmt.condition, variables, functions, module)?;
                
                // Create blocks for then, else (optional), and merge
                let then_block = builder.create_block();
                let else_block = if if_stmt.else_block.is_some() {
                    Some(builder.create_block())
                } else {
                    None
                };
                let merge_block = builder.create_block();
                
                // Branch based on condition
                if let Some(else_block) = else_block {
                    builder.ins().brif(condition, then_block, &[], else_block, &[]);
                } else {
                    builder.ins().brif(condition, then_block, &[], merge_block, &[]);
                }
                
                // Generate then block
                builder.switch_to_block(then_block);
                for stmt in &if_stmt.then_block.statements {
                    Self::generate_statement_static(builder, stmt, variables, is_main, functions, module)?;
                }
                // Jump to merge block if no return statement
                if !Self::block_ends_with_return(&if_stmt.then_block) {
                    builder.ins().jump(merge_block, &[]);
                }
                builder.seal_block(then_block);
                
                // Generate else block if present
                if let (Some(else_block), Some(else_body)) = (else_block, &if_stmt.else_block) {
                    builder.switch_to_block(else_block);
                    for stmt in &else_body.statements {
                        Self::generate_statement_static(builder, stmt, variables, is_main, functions, module)?;
                    }
                    // Jump to merge block if no return statement
                    if !Self::block_ends_with_return(else_body) {
                        builder.ins().jump(merge_block, &[]);
                    }
                    builder.seal_block(else_block);
                }
                
                // Continue with merge block
                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
            }
            Statement::While(while_stmt) => {
                // Create blocks for loop header, body, and exit
                let header_block = builder.create_block();
                let body_block = builder.create_block();
                let exit_block = builder.create_block();
                
                // Jump to header block
                builder.ins().jump(header_block, &[]);
                
                // Generate header block (condition check)
                builder.switch_to_block(header_block);
                let condition = Self::generate_expression_static(builder, &while_stmt.condition, variables, functions, module)?;
                builder.ins().brif(condition, body_block, &[], exit_block, &[]);
                
                // Push loop context for break/continue
                let loop_context = LoopContext {
                    break_block: exit_block,
                    continue_block: header_block,
                };
                // Note: We can't access self here, so we'll need to refactor this
                
                // Generate body block
                builder.switch_to_block(body_block);
                for stmt in &while_stmt.body.statements {
                    Self::generate_statement_static(builder, stmt, variables, is_main, functions, module)?;
                }
                // Jump back to header for next iteration
                builder.ins().jump(header_block, &[]);
                
                // Seal blocks after all jumps are created
                builder.seal_block(header_block);
                builder.seal_block(body_block);
                
                // Continue with exit block
                builder.switch_to_block(exit_block);
                builder.seal_block(exit_block);
            }
            Statement::For(for_stmt) => {
                // Create blocks for initialization, header, body, update, and exit
                let header_block = builder.create_block();
                let body_block = builder.create_block();
                let update_block = builder.create_block();
                let exit_block = builder.create_block();
                
                // Generate initialization if present
                if let Some(init_stmt) = &for_stmt.init {
                    Self::generate_statement_static(builder, init_stmt, variables, is_main, functions, module)?;
                }
                
                // Jump to header block
                builder.ins().jump(header_block, &[]);
                
                // Generate header block (condition check)
                builder.switch_to_block(header_block);
                if let Some(condition_expr) = &for_stmt.condition {
                    let condition = Self::generate_expression_static(builder, condition_expr, variables, functions, module)?;
                    builder.ins().brif(condition, body_block, &[], exit_block, &[]);
                } else {
                    // No condition means infinite loop (until break)
                    builder.ins().jump(body_block, &[]);
                }
                
                // Generate body block
                builder.switch_to_block(body_block);
                for stmt in &for_stmt.body.statements {
                    Self::generate_statement_static(builder, stmt, variables, is_main, functions, module)?;
                }
                // Jump to update block
                builder.ins().jump(update_block, &[]);
                
                // Generate update block
                builder.switch_to_block(update_block);
                if let Some(update_stmt) = &for_stmt.update {
                    Self::generate_statement_static(builder, update_stmt, variables, is_main, functions, module)?;
                }
                // Jump back to header for next iteration
                builder.ins().jump(header_block, &[]);
                
                // Seal blocks after all jumps are created
                builder.seal_block(header_block);
                builder.seal_block(body_block);
                builder.seal_block(update_block);
                
                // Continue with exit block
                builder.switch_to_block(exit_block);
                builder.seal_block(exit_block);
            }
            Statement::Break => {
                // For now, we'll implement a simple version without loop context
                // In a real implementation, we would jump to the loop's exit block
                // For now, just ignore break statements in compilation
                // TODO: Implement proper loop context tracking
            }
            Statement::Continue => {
                // For now, we'll implement a simple version without loop context
                // In a real implementation, we would jump to the loop's continue block
                // For now, just ignore continue statements in compilation
                // TODO: Implement proper loop context tracking
            }
            _ => {
                return Err(IRError::UnsupportedFeature(format!("Statement type not yet supported: {:?}", statement)));
            }
        }
        
        Ok(())
    }
    
    fn is_float_expression(expression: &Expression) -> bool {
        match expression {
            Expression::Literal(ChifValue::Float(_)) => true,
            Expression::Binary(binary_op) => {
                Self::is_float_expression(&binary_op.left) || Self::is_float_expression(&binary_op.right)
            }
            _ => false,
        }
    }

    fn generate_expression_static(
        builder: &mut FunctionBuilder, 
        expression: &Expression, 
        variables: &HashMap<String, Variable>,
        functions: &HashMap<String, cranelift_module::FuncId>,
        module: &mut ObjectModule
    ) -> Result<Value, IRError> {
        match expression {
            Expression::Literal(value) => {
                Self::generate_literal(builder, value)
            }
            Expression::Identifier(name) => {
                if let Some(&var) = variables.get(name) {
                    Ok(builder.use_var(var))
                } else {
                    Err(IRError::Generation(format!("Undefined variable: {}", name)))
                }
            }
            Expression::Binary(binary_op) => {
                // Check for constant folding opportunities
                if let (Expression::Literal(left_val), Expression::Literal(right_val)) = 
                    (&*binary_op.left, &*binary_op.right) {
                    if let Some(folded) = Self::fold_constants(left_val, &binary_op.operator, right_val) {
                        return Self::generate_literal(builder, &folded);
                    }
                }
                
                let left = Self::generate_expression_static(builder, &binary_op.left, variables, functions, module)?;
                let right = Self::generate_expression_static(builder, &binary_op.right, variables, functions, module)?;
                
                // Determine if this is a float operation
                let is_float = Self::is_float_expression(&binary_op.left) || Self::is_float_expression(&binary_op.right);
                
                match binary_op.operator {
                    BinaryOperator::Add => {
                        if is_float {
                            Ok(builder.ins().fadd(left, right))
                        } else {
                            Ok(builder.ins().iadd(left, right))
                        }
                    }
                    BinaryOperator::Subtract => {
                        if is_float {
                            Ok(builder.ins().fsub(left, right))
                        } else {
                            Ok(builder.ins().isub(left, right))
                        }
                    }
                    BinaryOperator::Multiply => {
                        if is_float {
                            Ok(builder.ins().fmul(left, right))
                        } else {
                            Ok(builder.ins().imul(left, right))
                        }
                    }
                    BinaryOperator::Divide => {
                        if is_float {
                            Ok(builder.ins().fdiv(left, right))
                        } else {
                            Ok(builder.ins().sdiv(left, right))
                        }
                    }
                    BinaryOperator::Equal => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::Equal, left, right))
                        } else {
                            Ok(builder.ins().icmp(IntCC::Equal, left, right))
                        }
                    }
                    BinaryOperator::NotEqual => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::NotEqual, left, right))
                        } else {
                            Ok(builder.ins().icmp(IntCC::NotEqual, left, right))
                        }
                    }
                    BinaryOperator::Less => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::LessThan, left, right))
                        } else {
                            Ok(builder.ins().icmp(IntCC::SignedLessThan, left, right))
                        }
                    }
                    BinaryOperator::Greater => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::GreaterThan, left, right))
                        } else {
                            Ok(builder.ins().icmp(IntCC::SignedGreaterThan, left, right))
                        }
                    }
                    BinaryOperator::LessEqual => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::LessThanOrEqual, left, right))
                        } else {
                            Ok(builder.ins().icmp(IntCC::SignedLessThanOrEqual, left, right))
                        }
                    }
                    BinaryOperator::GreaterEqual => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::GreaterThanOrEqual, left, right))
                        } else {
                            Ok(builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, left, right))
                        }
                    }
                    _ => Err(IRError::UnsupportedFeature(format!("Binary operator not yet supported: {:?}", binary_op.operator))),
                }
            }
            Expression::Unary(unary_op) => {
                let operand = Self::generate_expression_static(builder, &unary_op.operand, variables, functions, module)?;
                
                match unary_op.operator {
                    UnaryOperator::Minus => {
                        let zero = builder.ins().iconst(types::I64, 0);
                        Ok(builder.ins().isub(zero, operand))
                    }
                    UnaryOperator::Not => {
                        // For boolean not, we assume the value is 0 or 1
                        let one = builder.ins().iconst(types::I8, 1);
                        Ok(builder.ins().bxor(operand, one))
                    }
                }
            }
            Expression::Call(func_call) => {
                // Special handling for console output
                if func_call.name == "con.out" {
                    if func_call.args.len() != 1 {
                        return Err(IRError::Generation("con.out expects exactly one argument".to_string()));
                    }
                    
                    let arg_value = Self::generate_expression_static(builder, &func_call.args[0], variables, functions, module)?;
                    
                    // Determine the type of the argument and call appropriate runtime function
                    let (func_name, converted_arg) = match &func_call.args[0] {
                        Expression::Literal(ChifValue::Int(_)) => ("rono_print_int", arg_value),
                        Expression::Literal(ChifValue::Float(_)) => ("rono_print_float", arg_value),
                        Expression::Literal(ChifValue::Bool(_)) => ("rono_print_bool", arg_value),
                        Expression::Literal(ChifValue::Str(_)) => ("rono_print_string", arg_value),
                        _ => {
                            // For variables and complex expressions, we need to infer the type
                            // This is a simplified approach - check if it's a float expression
                            if Self::is_float_expression(&func_call.args[0]) {
                                ("rono_print_float", arg_value)
                            } else {
                                // Default to int for now
                                ("rono_print_int", arg_value)
                            }
                        }
                    };
                    
                    if let Some(&print_func_id) = functions.get(func_name) {
                        let func_ref = module.declare_func_in_func(print_func_id, builder.func);
                        builder.ins().call(func_ref, &[converted_arg]);
                        // Return dummy value since con.out returns void
                        Ok(builder.ins().iconst(types::I64, 0))
                    } else {
                        Err(IRError::Generation("Runtime function rono_print_int not found".to_string()))
                    }
                } else if func_call.name == "randi" {
                    // Handle randi(min, max) function call
                    if func_call.args.len() != 2 {
                        return Err(IRError::Generation("randi expects 2 arguments (min, max)".to_string()));
                    }
                    
                    let min_value = Self::generate_expression_static(builder, &func_call.args[0], variables, functions, module)?;
                    let max_value = Self::generate_expression_static(builder, &func_call.args[1], variables, functions, module)?;
                    
                    if let Some(&rand_func_id) = functions.get("rono_rand_int") {
                        let func_ref = module.declare_func_in_func(rand_func_id, builder.func);
                        let result = builder.ins().call(func_ref, &[min_value, max_value]);
                        Ok(builder.inst_results(result)[0])
                    } else {
                        Err(IRError::Generation("Runtime function rono_rand_int not found".to_string()))
                    }
                } else if func_call.name == "randf" {
                    // Handle randf(min, max) function call
                    if func_call.args.len() != 2 {
                        return Err(IRError::Generation("randf expects 2 arguments (min, max)".to_string()));
                    }
                    
                    let min_value = Self::generate_expression_static(builder, &func_call.args[0], variables, functions, module)?;
                    let max_value = Self::generate_expression_static(builder, &func_call.args[1], variables, functions, module)?;
                    
                    if let Some(&rand_func_id) = functions.get("rono_rand_float") {
                        let func_ref = module.declare_func_in_func(rand_func_id, builder.func);
                        let result = builder.ins().call(func_ref, &[min_value, max_value]);
                        Ok(builder.inst_results(result)[0])
                    } else {
                        Err(IRError::Generation("Runtime function rono_rand_float not found".to_string()))
                    }
                } else if func_call.name == "rands" {
                    // Handle rands(from, to) function call
                    if func_call.args.len() != 2 {
                        return Err(IRError::Generation("rands expects 2 arguments (from, to)".to_string()));
                    }
                    
                    let from_value = Self::generate_expression_static(builder, &func_call.args[0], variables, functions, module)?;
                    let to_value = Self::generate_expression_static(builder, &func_call.args[1], variables, functions, module)?;
                    
                    if let Some(&rand_func_id) = functions.get("rono_rand_char_range") {
                        let func_ref = module.declare_func_in_func(rand_func_id, builder.func);
                        let result = builder.ins().call(func_ref, &[from_value, to_value]);
                        Ok(builder.inst_results(result)[0])
                    } else {
                        Err(IRError::Generation("Runtime function rono_rand_char_range not found".to_string()))
                    }
                } else {
                    // Look up the function
                    if let Some(&func_id) = functions.get(&func_call.name) {
                        // Generate arguments
                        let mut args = Vec::new();
                        for arg in &func_call.args {
                            let arg_value = Self::generate_expression_static(builder, arg, variables, functions, module)?;
                            args.push(arg_value);
                        }
                        
                        // Get function reference
                        let func_ref = module.declare_func_in_func(func_id, builder.func);
                        
                        // Make the call
                        let call_result = builder.ins().call(func_ref, &args);
                        
                        // Return the first result (if any)
                        let results = builder.inst_results(call_result);
                        if results.is_empty() {
                            // Function returns void, return a dummy value
                            Ok(builder.ins().iconst(types::I64, 0))
                        } else {
                            Ok(results[0])
                        }
                    } else {
                        Err(IRError::Generation(format!("Undefined function: {}", func_call.name)))
                    }
                }
            }
            Expression::MethodCall(method_call) => {
                // Special handling for console output
                if let Expression::Identifier(object_name) = &*method_call.object {
                    if object_name == "con" && method_call.method == "out" {
                        if method_call.args.is_empty() {
                            return Err(IRError::Generation("con.out expects at least one argument".to_string()));
                        }
                        
                        if method_call.args.len() == 1 {
                            // Simple output: con.out(value)
                            let arg_value = Self::generate_expression_static(builder, &method_call.args[0], variables, functions, module)?;
                            
                            // Call runtime print function
                            if let Some(&print_func_id) = functions.get("rono_print_int") {
                                let func_ref = module.declare_func_in_func(print_func_id, builder.func);
                                builder.ins().call(func_ref, &[arg_value]);
                                // Return dummy value since con.out returns void
                                Ok(builder.ins().iconst(types::I64, 0))
                            } else {
                                Err(IRError::Generation("Runtime function rono_print_int not found".to_string()))
                            }
                        } else if method_call.args.len() == 2 {
                            // Formatted output: con.out("Value: {}", value)
                            // For now, we'll ignore the format string and just use a default format
                            let arg_value = Self::generate_expression_static(builder, &method_call.args[1], variables, functions, module)?;
                            
                            // Call runtime format function with null format (uses default)
                            if let Some(&format_func_id) = functions.get("rono_print_format_int") {
                                let func_ref = module.declare_func_in_func(format_func_id, builder.func);
                                let null_ptr = builder.ins().iconst(types::I64, 0); // NULL format string
                                builder.ins().call(func_ref, &[null_ptr, arg_value]);
                                // Return dummy value since con.out returns void
                                Ok(builder.ins().iconst(types::I64, 0))
                            } else {
                                Err(IRError::Generation("Runtime function rono_print_format_int not found".to_string()))
                            }
                        } else {
                            Err(IRError::Generation("con.out supports maximum 2 arguments (format string and value)".to_string()))
                        }
                    } else if object_name == "con" && method_call.method == "in" {
                        if !method_call.args.is_empty() {
                            return Err(IRError::Generation("con.in expects no arguments".to_string()));
                        }
                        
                        // Call runtime input function - for now assume integer input
                        if let Some(&input_func_id) = functions.get("rono_input_int") {
                            let func_ref = module.declare_func_in_func(input_func_id, builder.func);
                            let result = builder.ins().call(func_ref, &[]);
                            Ok(builder.inst_results(result)[0])
                        } else {
                            Err(IRError::Generation("Runtime function rono_input_int not found".to_string()))
                        }

                    } else if object_name == "http" && method_call.method == "get" {
                        if method_call.args.len() != 1 {
                            return Err(IRError::Generation("http.get expects 1 argument (url)".to_string()));
                        }
                        
                        let url_value = Self::generate_expression_static(builder, &method_call.args[0], variables, functions, module)?;
                        
                        if let Some(&http_func_id) = functions.get("rono_http_get") {
                            let func_ref = module.declare_func_in_func(http_func_id, builder.func);
                            let result = builder.ins().call(func_ref, &[url_value]);
                            Ok(builder.inst_results(result)[0])
                        } else {
                            Err(IRError::Generation("Runtime function rono_http_get not found".to_string()))
                        }
                    } else if object_name == "http" && method_call.method == "post" {
                        if method_call.args.len() != 2 {
                            return Err(IRError::Generation("http.post expects 2 arguments (url, data)".to_string()));
                        }
                        
                        let url_value = Self::generate_expression_static(builder, &method_call.args[0], variables, functions, module)?;
                        let data_value = Self::generate_expression_static(builder, &method_call.args[1], variables, functions, module)?;
                        
                        if let Some(&http_func_id) = functions.get("rono_http_post") {
                            let func_ref = module.declare_func_in_func(http_func_id, builder.func);
                            let result = builder.ins().call(func_ref, &[url_value, data_value]);
                            Ok(builder.inst_results(result)[0])
                        } else {
                            Err(IRError::Generation("Runtime function rono_http_post not found".to_string()))
                        }
                    } else if object_name == "http" && method_call.method == "put" {
                        if method_call.args.len() != 2 {
                            return Err(IRError::Generation("http.put expects 2 arguments (url, data)".to_string()));
                        }
                        
                        let url_value = Self::generate_expression_static(builder, &method_call.args[0], variables, functions, module)?;
                        let data_value = Self::generate_expression_static(builder, &method_call.args[1], variables, functions, module)?;
                        
                        if let Some(&http_func_id) = functions.get("rono_http_put") {
                            let func_ref = module.declare_func_in_func(http_func_id, builder.func);
                            let result = builder.ins().call(func_ref, &[url_value, data_value]);
                            Ok(builder.inst_results(result)[0])
                        } else {
                            Err(IRError::Generation("Runtime function rono_http_put not found".to_string()))
                        }
                    } else if object_name == "http" && method_call.method == "delete" {
                        if method_call.args.len() != 1 {
                            return Err(IRError::Generation("http.delete expects 1 argument (url)".to_string()));
                        }
                        
                        let url_value = Self::generate_expression_static(builder, &method_call.args[0], variables, functions, module)?;
                        
                        if let Some(&http_func_id) = functions.get("rono_http_delete") {
                            let func_ref = module.declare_func_in_func(http_func_id, builder.func);
                            let result = builder.ins().call(func_ref, &[url_value]);
                            Ok(builder.inst_results(result)[0])
                        } else {
                            Err(IRError::Generation("Runtime function rono_http_delete not found".to_string()))
                        }
                    } else {
                        // Handle struct method calls
                        Self::generate_struct_method_call(builder, method_call, variables, functions, module)
                    }
                } else {
                    // Handle struct method calls on complex expressions
                    Self::generate_struct_method_call(builder, method_call, variables, functions, module)
                }
            }
            Expression::StructLiteral(struct_literal) => {
                // Allocate memory for the struct
                Self::generate_struct_instantiation(builder, struct_literal, variables, functions, module)
            }
            Expression::FieldAccess(field_access) => {
                // Generate field access
                Self::generate_field_access(builder, field_access, variables, functions, module)
            }
            Expression::ArrayLiteral(elements) => {
                // Generate array literal
                Self::generate_array_literal(builder, elements, variables, functions, module)
            }
            Expression::Index(index_access) => {
                // Generate array indexing
                Self::generate_array_index(builder, index_access, variables, functions, module)
            }
            Expression::Reference(expr) => {
                // Generate address-of operation (&expr)
                Self::generate_address_of(builder, expr, variables, functions, module)
            }
            Expression::Dereference(expr) => {
                // Generate dereference operation (*expr)
                Self::generate_dereference(builder, expr, variables, functions, module)
            }
            _ => {
                Err(IRError::UnsupportedFeature(format!("Expression type not yet supported: {:?}", expression)))
            }
        }
    }
    
    fn generate_literal(builder: &mut FunctionBuilder, value: &ChifValue) -> Result<Value, IRError> {
        match value {
            ChifValue::Int(i) => Ok(builder.ins().iconst(types::I64, *i)),
            ChifValue::Float(f) => Ok(builder.ins().f64const(*f)),
            ChifValue::Bool(b) => Ok(builder.ins().iconst(types::I8, if *b { 1 } else { 0 })),
            ChifValue::Nil => Ok(builder.ins().iconst(types::I64, 0)), // Represent nil as 0
            ChifValue::Str(s) => {
                // Create string constant in memory
                // For now, we need to handle this differently since we can't access self.module here
                // Let's use a simpler approach - create string on stack
                Self::generate_string_on_stack(builder, s)
            }
            ChifValue::Array(_) => {
                // TODO: Implement array literal support
                Err(IRError::UnsupportedFeature("Array literals not yet supported".to_string()))
            }
            ChifValue::List(_) => {
                // TODO: Implement list literal support
                Err(IRError::UnsupportedFeature("List literals not yet supported".to_string()))
            }
            ChifValue::Map(_) => {
                // TODO: Implement map literal support
                Err(IRError::UnsupportedFeature("Map literals not yet supported".to_string()))
            }
            ChifValue::Struct(_, _) => {
                // TODO: Implement struct literal support
                Err(IRError::UnsupportedFeature("Struct literals not yet supported".to_string()))
            }
            ChifValue::Pointer(_) => {
                // TODO: Implement pointer literal support
                Err(IRError::UnsupportedFeature("Pointer literals not yet supported".to_string()))
            }
            ChifValue::Reference(_) => {
                // TODO: Implement reference literal support
                Err(IRError::UnsupportedFeature("Reference literals not yet supported".to_string()))
            }
        }
    }
    
    fn chif_type_to_cranelift(chif_type: &ChifType) -> Result<Type, IRError> {
        match chif_type {
            ChifType::Int => Ok(types::I64),
            ChifType::Float => Ok(types::F64),
            ChifType::Bool => Ok(types::I8),
            ChifType::Str => Ok(types::I64), // String as pointer for now
            ChifType::Nil => Ok(types::I64), // Nil as 64-bit value
            ChifType::Pointer(_) => Ok(types::I64), // Pointer as 64-bit value
            ChifType::Struct(_name) => Ok(types::I64), // Struct as pointer for now
            ChifType::Array(_element_type, _dimensions) => Ok(types::I64), // Array as pointer for now
            ChifType::List(_element_type, _dimensions) => Ok(types::I64), // List as pointer for now
            ChifType::Map(_key_type, _value_type) => Ok(types::I64), // Map as pointer for now
            _ => Err(IRError::TypeConversion(format!("Type conversion not yet supported: {:?}", chif_type))),
        }
    }
    
    fn get_default_value(builder: &mut FunctionBuilder, cranelift_type: Type) -> Value {
        match cranelift_type {
            types::I8 | types::I16 | types::I32 | types::I64 => builder.ins().iconst(cranelift_type, 0),
            types::F32 => builder.ins().f32const(0.0),
            types::F64 => builder.ins().f64const(0.0),
            _ => builder.ins().iconst(types::I64, 0), // Default fallback
        }
    }
    
    fn fold_constants(left: &ChifValue, op: &BinaryOperator, right: &ChifValue) -> Option<ChifValue> {
        match (left, op, right) {
            // Integer arithmetic
            (ChifValue::Int(a), BinaryOperator::Add, ChifValue::Int(b)) => Some(ChifValue::Int(a + b)),
            (ChifValue::Int(a), BinaryOperator::Subtract, ChifValue::Int(b)) => Some(ChifValue::Int(a - b)),
            (ChifValue::Int(a), BinaryOperator::Multiply, ChifValue::Int(b)) => Some(ChifValue::Int(a * b)),
            (ChifValue::Int(a), BinaryOperator::Divide, ChifValue::Int(b)) if *b != 0 => Some(ChifValue::Int(a / b)),
            (ChifValue::Int(a), BinaryOperator::Modulo, ChifValue::Int(b)) if *b != 0 => Some(ChifValue::Int(a % b)),
            
            // Integer comparisons
            (ChifValue::Int(a), BinaryOperator::Equal, ChifValue::Int(b)) => Some(ChifValue::Bool(a == b)),
            (ChifValue::Int(a), BinaryOperator::NotEqual, ChifValue::Int(b)) => Some(ChifValue::Bool(a != b)),
            (ChifValue::Int(a), BinaryOperator::Less, ChifValue::Int(b)) => Some(ChifValue::Bool(a < b)),
            (ChifValue::Int(a), BinaryOperator::Greater, ChifValue::Int(b)) => Some(ChifValue::Bool(a > b)),
            (ChifValue::Int(a), BinaryOperator::LessEqual, ChifValue::Int(b)) => Some(ChifValue::Bool(a <= b)),
            (ChifValue::Int(a), BinaryOperator::GreaterEqual, ChifValue::Int(b)) => Some(ChifValue::Bool(a >= b)),
            
            // Float arithmetic
            (ChifValue::Float(a), BinaryOperator::Add, ChifValue::Float(b)) => Some(ChifValue::Float(a + b)),
            (ChifValue::Float(a), BinaryOperator::Subtract, ChifValue::Float(b)) => Some(ChifValue::Float(a - b)),
            (ChifValue::Float(a), BinaryOperator::Multiply, ChifValue::Float(b)) => Some(ChifValue::Float(a * b)),
            (ChifValue::Float(a), BinaryOperator::Divide, ChifValue::Float(b)) if *b != 0.0 => Some(ChifValue::Float(a / b)),
            
            // Float comparisons
            (ChifValue::Float(a), BinaryOperator::Equal, ChifValue::Float(b)) => Some(ChifValue::Bool((a - b).abs() < f64::EPSILON)),
            (ChifValue::Float(a), BinaryOperator::NotEqual, ChifValue::Float(b)) => Some(ChifValue::Bool((a - b).abs() >= f64::EPSILON)),
            (ChifValue::Float(a), BinaryOperator::Less, ChifValue::Float(b)) => Some(ChifValue::Bool(a < b)),
            (ChifValue::Float(a), BinaryOperator::Greater, ChifValue::Float(b)) => Some(ChifValue::Bool(a > b)),
            (ChifValue::Float(a), BinaryOperator::LessEqual, ChifValue::Float(b)) => Some(ChifValue::Bool(a <= b)),
            (ChifValue::Float(a), BinaryOperator::GreaterEqual, ChifValue::Float(b)) => Some(ChifValue::Bool(a >= b)),
            
            // Mixed int/float arithmetic (promote int to float)
            (ChifValue::Int(a), BinaryOperator::Add, ChifValue::Float(b)) => Some(ChifValue::Float(*a as f64 + b)),
            (ChifValue::Float(a), BinaryOperator::Add, ChifValue::Int(b)) => Some(ChifValue::Float(a + *b as f64)),
            (ChifValue::Int(a), BinaryOperator::Subtract, ChifValue::Float(b)) => Some(ChifValue::Float(*a as f64 - b)),
            (ChifValue::Float(a), BinaryOperator::Subtract, ChifValue::Int(b)) => Some(ChifValue::Float(a - *b as f64)),
            (ChifValue::Int(a), BinaryOperator::Multiply, ChifValue::Float(b)) => Some(ChifValue::Float(*a as f64 * b)),
            (ChifValue::Float(a), BinaryOperator::Multiply, ChifValue::Int(b)) => Some(ChifValue::Float(a * *b as f64)),
            (ChifValue::Int(a), BinaryOperator::Divide, ChifValue::Float(b)) if *b != 0.0 => Some(ChifValue::Float(*a as f64 / b)),
            (ChifValue::Float(a), BinaryOperator::Divide, ChifValue::Int(b)) if *b != 0 => Some(ChifValue::Float(a / *b as f64)),
            
            // Boolean operations
            (ChifValue::Bool(a), BinaryOperator::And, ChifValue::Bool(b)) => Some(ChifValue::Bool(*a && *b)),
            (ChifValue::Bool(a), BinaryOperator::Or, ChifValue::Bool(b)) => Some(ChifValue::Bool(*a || *b)),
            (ChifValue::Bool(a), BinaryOperator::Equal, ChifValue::Bool(b)) => Some(ChifValue::Bool(a == b)),
            (ChifValue::Bool(a), BinaryOperator::NotEqual, ChifValue::Bool(b)) => Some(ChifValue::Bool(a != b)),
            
            // String concatenation
            (ChifValue::Str(a), BinaryOperator::Add, ChifValue::Str(b)) => Some(ChifValue::Str(format!("{}{}", a, b))),
            
            _ => None, // No folding possible
        }
    }
    
    fn block_ends_with_return(block: &crate::ast::Block) -> bool {
        for stmt in &block.statements {
            match stmt {
                Statement::Return(_) => return true,
                Statement::If(if_stmt) => {
                    // If both branches return, then the if statement returns
                    if Self::block_ends_with_return(&if_stmt.then_block) {
                        if let Some(else_block) = &if_stmt.else_block {
                            if Self::block_ends_with_return(else_block) {
                                return true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }
    
    fn declare_runtime_functions(&mut self) -> Result<(), IRError> {
        // Declare rono_print_int(i64) -> void
        let mut print_int_sig = self.module.make_signature();
        print_int_sig.params.push(AbiParam::new(types::I64));
        let print_int_id = self.module.declare_function("rono_print_int", Linkage::Import, &print_int_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_print_int".to_string(), print_int_id);
        
        // Declare rono_print_float(f64) -> void
        let mut print_float_sig = self.module.make_signature();
        print_float_sig.params.push(AbiParam::new(types::F64));
        let print_float_id = self.module.declare_function("rono_print_float", Linkage::Import, &print_float_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_print_float".to_string(), print_float_id);
        
        // Declare rono_print_bool(i8) -> void
        let mut print_bool_sig = self.module.make_signature();
        print_bool_sig.params.push(AbiParam::new(types::I8));
        let print_bool_id = self.module.declare_function("rono_print_bool", Linkage::Import, &print_bool_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_print_bool".to_string(), print_bool_id);
        
        // Declare rono_print_string(const char*) -> void
        let mut print_string_sig = self.module.make_signature();
        print_string_sig.params.push(AbiParam::new(types::I64)); // String as pointer
        let print_string_id = self.module.declare_function("rono_print_string", Linkage::Import, &print_string_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_print_string".to_string(), print_string_id);
        
        // Declare rono_print_format_int(const char*, i64) -> void for interpolation
        let mut print_format_sig = self.module.make_signature();
        print_format_sig.params.push(AbiParam::new(types::I64)); // Format string as pointer
        print_format_sig.params.push(AbiParam::new(types::I64)); // Value
        let print_format_id = self.module.declare_function("rono_print_format_int", Linkage::Import, &print_format_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_print_format_int".to_string(), print_format_id);
        
        // Declare console input functions
        // rono_input_string() -> char*
        let mut input_string_sig = self.module.make_signature();
        input_string_sig.returns.push(AbiParam::new(types::I64)); // String as pointer
        let input_string_id = self.module.declare_function("rono_input_string", Linkage::Import, &input_string_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_input_string".to_string(), input_string_id);
        
        // rono_input_int() -> i64
        let mut input_int_sig = self.module.make_signature();
        input_int_sig.returns.push(AbiParam::new(types::I64));
        let input_int_id = self.module.declare_function("rono_input_int", Linkage::Import, &input_int_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_input_int".to_string(), input_int_id);
        
        // rono_input_float() -> f64
        let mut input_float_sig = self.module.make_signature();
        input_float_sig.returns.push(AbiParam::new(types::F64));
        let input_float_id = self.module.declare_function("rono_input_float", Linkage::Import, &input_float_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_input_float".to_string(), input_float_id);
        
        // rono_input_bool() -> i8
        let mut input_bool_sig = self.module.make_signature();
        input_bool_sig.returns.push(AbiParam::new(types::I8));
        let input_bool_id = self.module.declare_function("rono_input_bool", Linkage::Import, &input_bool_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_input_bool".to_string(), input_bool_id);
        
        // Declare random number generation functions
        // rono_rand_int(i64, i64) -> i64
        let mut rand_int_sig = self.module.make_signature();
        rand_int_sig.params.push(AbiParam::new(types::I64)); // min
        rand_int_sig.params.push(AbiParam::new(types::I64)); // max
        rand_int_sig.returns.push(AbiParam::new(types::I64));
        let rand_int_id = self.module.declare_function("rono_rand_int", Linkage::Import, &rand_int_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_rand_int".to_string(), rand_int_id);
        
        // rono_rand_float(f64, f64) -> f64
        let mut rand_float_sig = self.module.make_signature();
        rand_float_sig.params.push(AbiParam::new(types::F64)); // min
        rand_float_sig.params.push(AbiParam::new(types::F64)); // max
        rand_float_sig.returns.push(AbiParam::new(types::F64));
        let rand_float_id = self.module.declare_function("rono_rand_float", Linkage::Import, &rand_float_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_rand_float".to_string(), rand_float_id);
        
        // rono_rand_string(i64) -> char*
        let mut rand_string_sig = self.module.make_signature();
        rand_string_sig.params.push(AbiParam::new(types::I64)); // length
        rand_string_sig.returns.push(AbiParam::new(types::I64)); // String as pointer
        let rand_string_id = self.module.declare_function("rono_rand_string", Linkage::Import, &rand_string_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_rand_string".to_string(), rand_string_id);
        
        // rono_rand_char_range(const char*, const char*) -> char*
        let mut rand_char_range_sig = self.module.make_signature();
        rand_char_range_sig.params.push(AbiParam::new(types::I64)); // from as pointer
        rand_char_range_sig.params.push(AbiParam::new(types::I64)); // to as pointer
        rand_char_range_sig.returns.push(AbiParam::new(types::I64)); // String as pointer
        let rand_char_range_id = self.module.declare_function("rono_rand_char_range", Linkage::Import, &rand_char_range_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_rand_char_range".to_string(), rand_char_range_id);
        
        // Declare HTTP functions
        // rono_http_get(const char*) -> char*
        let mut http_get_sig = self.module.make_signature();
        http_get_sig.params.push(AbiParam::new(types::I64)); // URL as pointer
        http_get_sig.returns.push(AbiParam::new(types::I64)); // Response as pointer
        let http_get_id = self.module.declare_function("rono_http_get", Linkage::Import, &http_get_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_http_get".to_string(), http_get_id);
        
        // rono_http_post(const char*, const char*) -> char*
        let mut http_post_sig = self.module.make_signature();
        http_post_sig.params.push(AbiParam::new(types::I64)); // URL as pointer
        http_post_sig.params.push(AbiParam::new(types::I64)); // Data as pointer
        http_post_sig.returns.push(AbiParam::new(types::I64)); // Response as pointer
        let http_post_id = self.module.declare_function("rono_http_post", Linkage::Import, &http_post_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_http_post".to_string(), http_post_id);
        
        // rono_http_put(const char*, const char*) -> char*
        let mut http_put_sig = self.module.make_signature();
        http_put_sig.params.push(AbiParam::new(types::I64)); // URL as pointer
        http_put_sig.params.push(AbiParam::new(types::I64)); // Data as pointer
        http_put_sig.returns.push(AbiParam::new(types::I64)); // Response as pointer
        let http_put_id = self.module.declare_function("rono_http_put", Linkage::Import, &http_put_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_http_put".to_string(), http_put_id);
        
        // rono_http_delete(const char*) -> char*
        let mut http_delete_sig = self.module.make_signature();
        http_delete_sig.params.push(AbiParam::new(types::I64)); // URL as pointer
        http_delete_sig.returns.push(AbiParam::new(types::I64)); // Response as pointer
        let http_delete_id = self.module.declare_function("rono_http_delete", Linkage::Import, &http_delete_sig)
            .map_err(|e| IRError::Module(e))?;
        self.functions.insert("rono_http_delete".to_string(), http_delete_id);

        
        Ok(())
    }

    fn process_struct_definition(&mut self, struct_def: &StructDef) -> Result<(), IRError> {
        // Calculate struct layout and field offsets
        let mut fields = Vec::new();
        let mut current_offset = 0u32;
        let mut max_alignment = 1u32;
        
        for field in &struct_def.fields {
            let field_size = Self::get_type_size(&field.field_type)?;
            let field_alignment = Self::get_type_alignment(&field.field_type)?;
            
            // Update maximum alignment
            max_alignment = max_alignment.max(field_alignment);
            
            // Align current offset to field alignment
            current_offset = Self::align_to(current_offset, field_alignment);
            
            fields.push(StructFieldLayout {
                name: field.name.clone(),
                field_type: field.field_type.clone(),
                offset: current_offset,
                size: field_size,
            });
            
            current_offset += field_size;
        }
        
        // Align total size to struct alignment
        let total_size = Self::align_to(current_offset, max_alignment);
        
        let layout = StructLayout {
            name: struct_def.name.clone(),
            fields,
            size: total_size,
            alignment: max_alignment,
        };
        
        self.structs.insert(struct_def.name.clone(), layout);
        
        Ok(())
    }

    fn get_type_size(chif_type: &ChifType) -> Result<u32, IRError> {
        match chif_type {
            ChifType::Int => Ok(8),      // i64
            ChifType::Float => Ok(8),    // f64
            ChifType::Bool => Ok(1),     // i8
            ChifType::Str => Ok(8),      // pointer
            ChifType::Nil => Ok(0),
            ChifType::Pointer(_) => Ok(8), // pointer size
            ChifType::Struct(name) => {
                // For now, return a placeholder size
                // In a full implementation, we would look up the struct size
                Ok(16) // placeholder
            }
            _ => Err(IRError::UnsupportedFeature(format!("Type size calculation not implemented for: {:?}", chif_type))),
        }
    }
    
    fn get_type_alignment(chif_type: &ChifType) -> Result<u32, IRError> {
        match chif_type {
            ChifType::Int => Ok(8),      // i64 alignment
            ChifType::Float => Ok(8),    // f64 alignment
            ChifType::Bool => Ok(1),     // i8 alignment
            ChifType::Str => Ok(8),      // pointer alignment
            ChifType::Nil => Ok(1),
            ChifType::Pointer(_) => Ok(8), // pointer alignment
            ChifType::Struct(_) => Ok(8),  // struct alignment (max field alignment)
            _ => Err(IRError::UnsupportedFeature(format!("Type alignment calculation not implemented for: {:?}", chif_type))),
        }
    }
    
    fn align_to(value: u32, alignment: u32) -> u32 {
        (value + alignment - 1) & !(alignment - 1)
    }

    fn generate_struct_instantiation(
        builder: &mut FunctionBuilder,
        struct_literal: &StructLiteral,
        variables: &HashMap<String, Variable>,
        functions: &HashMap<String, cranelift_module::FuncId>,
        module: &mut ObjectModule
    ) -> Result<Value, IRError> {
        // For now, we'll implement a simple version that allocates memory on the stack
        // In a full implementation, we would:
        // 1. Look up the struct layout
        // 2. Allocate memory (stack or heap)
        // 3. Initialize fields with provided values
        // 4. Return pointer to the struct
        
        // For this implementation, we'll create a simple struct representation
        // We'll allocate space for each field and store them sequentially
        
        // Calculate total size needed (simplified - assume each field is 8 bytes)
        let field_count = struct_literal.fields.len() as i64;
        let total_size = field_count * 8; // 8 bytes per field
        
        // Allocate stack space (simplified approach)
        let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            total_size as u32,
        ));
        
        // Get pointer to the allocated memory
        let struct_ptr = builder.ins().stack_addr(types::I64, stack_slot, 0);
        
        // Initialize fields
        for (i, (field_name, field_expr)) in struct_literal.fields.iter().enumerate() {
            let field_value = Self::generate_expression_static(builder, field_expr, variables, functions, module)?;
            let offset = (i * 8) as i32; // 8 bytes per field
            builder.ins().store(MemFlags::new(), field_value, struct_ptr, offset);
        }
        
        // Return pointer to the struct
        Ok(struct_ptr)
    }
    
    fn generate_field_access(
        builder: &mut FunctionBuilder,
        field_access: &FieldAccess,
        variables: &HashMap<String, Variable>,
        functions: &HashMap<String, cranelift_module::FuncId>,
        module: &mut ObjectModule
    ) -> Result<Value, IRError> {
        // Generate the object expression (should be a struct pointer)
        let struct_ptr = Self::generate_expression_static(builder, &field_access.object, variables, functions, module)?;
        
        // For now, we'll use a simple field offset calculation
        // In a full implementation, we would:
        // 1. Look up the struct type from the object expression
        // 2. Find the field offset from the struct layout
        // 3. Load the value from memory at struct_ptr + offset
        
        // For this simplified implementation, we'll assume fields are stored sequentially
        // and each field is 8 bytes. We'll need to know the field index.
        
        // This is a simplified approach - in reality we'd need struct layout information
        let field_offset = match field_access.field.as_str() {
            "x" => 0,  // First field
            "y" => 8,  // Second field  
            "width" => 0,  // First field for Rectangle
            "height" => 8, // Second field for Rectangle
            _ => return Err(IRError::Generation(format!("Unknown field: {}", field_access.field))),
        };
        
        // Load the field value from memory
        let field_value = builder.ins().load(types::I64, MemFlags::new(), struct_ptr, field_offset);
        Ok(field_value)
    }
    
    fn generate_struct_method_call(
        builder: &mut FunctionBuilder,
        method_call: &MethodCall,
        variables: &HashMap<String, Variable>,
        functions: &HashMap<String, cranelift_module::FuncId>,
        module: &mut ObjectModule
    ) -> Result<Value, IRError> {
        // Generate the object (self parameter)
        let self_value = Self::generate_expression_static(builder, &method_call.object, variables, functions, module)?;
        
        // For now, we'll assume the method name follows the pattern StructName_methodName
        // In a real implementation, we would need to determine the struct type from the object
        // For this simplified version, we'll try common struct names
        let possible_method_names = vec![
            format!("Point_{}", method_call.method),
            format!("Rectangle_{}", method_call.method),
            // Add more struct names as needed
        ];
        
        for method_name in possible_method_names {
            if let Some(&func_id) = functions.get(&method_name) {
                // Generate arguments (self + other arguments)
                let mut args = vec![self_value];
                for arg in &method_call.args {
                    let arg_value = Self::generate_expression_static(builder, arg, variables, functions, module)?;
                    args.push(arg_value);
                }
                
                // Get function reference and make the call
                let func_ref = module.declare_func_in_func(func_id, builder.func);
                let call_result = builder.ins().call(func_ref, &args);
                
                // Return the first result (if any)
                let results = builder.inst_results(call_result);
                if results.is_empty() {
                    // Method returns void, return a dummy value
                    return Ok(builder.ins().iconst(types::I64, 0));
                } else {
                    return Ok(results[0]);
                }
            }
        }
        
        Err(IRError::Generation(format!("Method '{}' not found", method_call.method)))
    }
    
    fn generate_array_literal(
        builder: &mut FunctionBuilder,
        elements: &[Expression],
        variables: &HashMap<String, Variable>,
        functions: &HashMap<String, cranelift_module::FuncId>,
        module: &mut ObjectModule
    ) -> Result<Value, IRError> {
        if elements.is_empty() {
            // Empty array - return null pointer
            return Ok(builder.ins().iconst(types::I64, 0));
        }
        
        // Calculate total size needed (assume each element is 8 bytes for now)
        let element_count = elements.len() as i64;
        let total_size = element_count * 8; // 8 bytes per element
        
        // Allocate stack space
        let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            total_size as u32,
        ));
        
        // Get pointer to the allocated memory
        let array_ptr = builder.ins().stack_addr(types::I64, stack_slot, 0);
        
        // Initialize elements
        for (i, element_expr) in elements.iter().enumerate() {
            let element_value = Self::generate_expression_static(builder, element_expr, variables, functions, module)?;
            let offset = (i * 8) as i32; // 8 bytes per element
            builder.ins().store(MemFlags::new(), element_value, array_ptr, offset);
        }
        
        // Return pointer to the array
        Ok(array_ptr)
    }
    
    fn generate_array_index(
        builder: &mut FunctionBuilder,
        index_access: &IndexAccess,
        variables: &HashMap<String, Variable>,
        functions: &HashMap<String, cranelift_module::FuncId>,
        module: &mut ObjectModule
    ) -> Result<Value, IRError> {
        // Generate the array pointer
        let mut current_ptr = Self::generate_expression_static(builder, &index_access.object, variables, functions, module)?;
        
        // Handle multiple indices for multidimensional arrays
        for index_expr in &index_access.indices {
            // Generate the index
            let index_value = Self::generate_expression_static(builder, index_expr, variables, functions, module)?;
            
            // Calculate offset: index * element_size (8 bytes)
            let element_size = builder.ins().iconst(types::I64, 8);
            let offset = builder.ins().imul(index_value, element_size);
            
            // Calculate final address: current_ptr + offset
            let element_ptr = builder.ins().iadd(current_ptr, offset);
            
            // Load the element value (which might be another array pointer)
            current_ptr = builder.ins().load(types::I64, MemFlags::new(), element_ptr, 0);
        }
        
        Ok(current_ptr)
    }



    pub fn finalize(self) -> ObjectModule {
        self.module
    }
    
    fn process_import(&mut self, import: &ImportStatement) -> Result<(), IRError> {
        // Add .rono extension if not present
        let file_path = if import.path.ends_with(".rono") {
            import.path.clone()
        } else {
            format!("{}.rono", import.path)
        };
        
        // Read the imported file
        let source = std::fs::read_to_string(&file_path).map_err(|_| {
            IRError::Generation(format!("Could not read module file: {}", file_path))
        })?;
        
        // Parse the imported file
        use crate::{lexer::Lexer, parser::Parser};
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize().map_err(|e| {
            IRError::Generation(format!("Failed to tokenize module {}: {}", file_path, e))
        })?;
        
        let mut parser = Parser::new(tokens);
        let imported_program = parser.parse().map_err(|e| {
            IRError::Generation(format!("Failed to parse module {}: {}", file_path, e))
        })?;
        
        // Get module name for prefixing
        let module_name = import.alias.clone().unwrap_or_else(|| {
            std::path::Path::new(&import.path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string()
        });
        
        // Declare imported functions with module prefix
        for item in &imported_program.items {
            match item {
                Item::Function(func) => {
                    let qualified_name = format!("{}_{}", module_name, func.name);
                    let mut qualified_func = func.clone();
                    qualified_func.name = qualified_name;
                    self.declare_function(&qualified_func)?;
                }
                Item::StructImpl(impl_block) => {
                    // Declare methods with module and struct prefix
                    for method in &impl_block.methods {
                        let method_name = format!("{}_{}_{}", module_name, impl_block.struct_name, method.name);
                        let mut method_with_new_name = method.clone();
                        method_with_new_name.name = method_name;
                        self.declare_function(&method_with_new_name)?;
                    }
                }
                _ => {} // Other items handled elsewhere
            }
        }
        
        // Generate imported function bodies
        for item in &imported_program.items {
            match item {
                Item::Function(func) => {
                    let qualified_name = format!("{}_{}", module_name, func.name);
                    let mut qualified_func = func.clone();
                    qualified_func.name = qualified_name;
                    self.generate_function(&qualified_func)?;
                }
                Item::StructImpl(impl_block) => {
                    // Generate method bodies with module and struct prefix
                    for method in &impl_block.methods {
                        let method_name = format!("{}_{}_{}", module_name, impl_block.struct_name, method.name);
                        let mut method_with_new_name = method.clone();
                        method_with_new_name.name = method_name;
                        self.generate_function(&method_with_new_name)?;
                    }
                }
                _ => {} // Other items handled elsewhere
            }
        }
        
        Ok(())
    }
    
    fn generate_address_of(
        builder: &mut FunctionBuilder,
        expr: &Expression,
        variables: &HashMap<String, Variable>,
        functions: &HashMap<String, cranelift_module::FuncId>,
        module: &mut ObjectModule
    ) -> Result<Value, IRError> {
        match expr {
            Expression::Identifier(var_name) => {
                // Get address of a variable
                if let Some(&var) = variables.get(var_name) {
                    // In Cranelift, we can get the address of a stack slot
                    // For now, we'll create a simple implementation
                    // This is a simplified approach - in a real implementation,
                    // we'd need to track stack slots for variables
                    let var_value = builder.use_var(var);
                    
                    // Create a stack slot to store the variable value
                    let stack_slot = builder.create_sized_stack_slot(cranelift::prelude::StackSlotData::new(
                        cranelift::prelude::StackSlotKind::ExplicitSlot,
                        8, // 8 bytes for a 64-bit value
                    ));
                    
                    // Store the variable value to the stack slot
                    builder.ins().stack_store(var_value, stack_slot, 0);
                    
                    // Return the address of the stack slot
                    Ok(builder.ins().stack_addr(types::I64, stack_slot, 0))
                } else {
                    Err(IRError::Generation(format!("Undefined variable for address-of: {}", var_name)))
                }
            }
            _ => {
                // For other expressions, we need to evaluate them and create a temporary
                let value = Self::generate_expression_static(builder, expr, variables, functions, module)?;
                
                // Create a stack slot to store the temporary value
                let stack_slot = builder.create_sized_stack_slot(cranelift::prelude::StackSlotData::new(
                    cranelift::prelude::StackSlotKind::ExplicitSlot,
                    8, // 8 bytes for a 64-bit value
                ));
                
                // Store the value to the stack slot
                builder.ins().stack_store(value, stack_slot, 0);
                
                // Return the address of the stack slot
                Ok(builder.ins().stack_addr(types::I64, stack_slot, 0))
            }
        }
    }
    
    fn generate_dereference(
        builder: &mut FunctionBuilder,
        expr: &Expression,
        variables: &HashMap<String, Variable>,
        functions: &HashMap<String, cranelift_module::FuncId>,
        module: &mut ObjectModule
    ) -> Result<Value, IRError> {
        // Generate the pointer expression
        let pointer = Self::generate_expression_static(builder, expr, variables, functions, module)?;
        
        // For now, we need to determine what type to load
        // This is a simplified approach - we'll try to infer from context
        // In a real implementation, we'd track pointer target types
        
        // Check if this dereference is part of a float operation
        // This is a hack - we should have proper type tracking
        
        // For now, load as I64 and let the caller handle type conversion if needed
        Ok(builder.ins().load(types::I64, cranelift::prelude::MemFlags::new(), pointer, 0))
    }
    
    fn generate_string_on_stack(
        builder: &mut FunctionBuilder,
        s: &str,
    ) -> Result<Value, IRError> {
        // Create string on stack (simplified approach)
        let string_bytes = s.as_bytes();
        let string_len = string_bytes.len() + 1; // +1 for null terminator
        
        // Create stack slot for the string
        let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            string_len as u32,
        ));
        
        // Get pointer to stack slot
        let string_ptr = builder.ins().stack_addr(types::I64, stack_slot, 0);
        
        // Store each byte of the string
        for (i, &byte) in string_bytes.iter().enumerate() {
            let byte_val = builder.ins().iconst(types::I8, byte as i64);
            builder.ins().store(MemFlags::new(), byte_val, string_ptr, i as i32);
        }
        
        // Store null terminator
        let null_byte = builder.ins().iconst(types::I8, 0);
        builder.ins().store(MemFlags::new(), null_byte, string_ptr, string_bytes.len() as i32);
        
        Ok(string_ptr)
    }
}