use crate::ast::*;
use crate::error::{ChifError, Result};
use crate::types::ChifValue;
use rand::Rng;
use std::collections::HashMap;
use std::io;

pub struct Interpreter {
    globals: HashMap<String, ChifValue>,
    locals: Vec<HashMap<String, ChifValue>>,
    functions: HashMap<String, Function>,
    structs: HashMap<String, StructDef>,
    struct_methods: HashMap<String, Vec<Function>>,
    modules: HashMap<String, Module>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub functions: HashMap<String, Function>,
    pub structs: HashMap<String, StructDef>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        
        // Add console object
        let mut console_methods = HashMap::new();
        console_methods.insert("out".to_string(), ChifValue::Str("console_out".to_string()));
        console_methods.insert("in".to_string(), ChifValue::Str("console_in".to_string()));
        globals.insert("con".to_string(), ChifValue::Struct("Console".to_string(), console_methods));
        
        Self {
            globals,
            locals: Vec::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            struct_methods: HashMap::new(),
            modules: HashMap::new(),
        }
    }
    
    pub fn execute(&mut self, program: &Program) -> Result<()> {
        // First pass: process imports and collect all functions and structs
        for item in &program.items {
            match item {
                Item::Import(import) => {
                    self.process_import(import)?;
                }
                Item::Function(func) => {
                    self.functions.insert(func.name.clone(), func.clone());
                }
                Item::Struct(struct_def) => {
                    self.structs.insert(struct_def.name.clone(), struct_def.clone());
                }
                Item::StructImpl(impl_block) => {
                    self.struct_methods
                        .entry(impl_block.struct_name.clone())
                        .or_insert_with(Vec::new)
                        .extend(impl_block.methods.clone());
                }
            }
        }
        
        // Find and execute main function
        if let Some(main_func) = self.functions.get("main").cloned() {
            if main_func.is_main {
                self.call_function(&main_func, Vec::new())?;
            } else {
                return Err(ChifError::RuntimeError {
                    message: "Main function must be marked with 'chif'".to_string(),
                });
            }
        } else {
            return Err(ChifError::RuntimeError {
                message: "No main function found".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn call_function(&mut self, func: &Function, args: Vec<ChifValue>) -> Result<ChifValue> {
        if args.len() != func.params.len() {
            return Err(ChifError::RuntimeError {
                message: format!(
                    "Function '{}' expects {} arguments, got {}",
                    func.name,
                    func.params.len(),
                    args.len()
                ),
            });
        }
        
        // Create new scope
        let mut scope = HashMap::new();
        
        // Bind parameters
        for (param, arg) in func.params.iter().zip(args.iter()) {
            scope.insert(param.name.clone(), arg.clone());
        }
        
        self.locals.push(scope);
        
        let result = self.execute_block(&func.body);
        
        self.locals.pop();
        
        match result {
            Ok(_) => Ok(ChifValue::Nil),
            Err(ChifError::Return(value)) => Ok(value),
            Err(e) => Err(e),
        }
    }
    
    fn execute_block(&mut self, block: &Block) -> Result<()> {
        for statement in &block.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }
    
    fn execute_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::VarDecl(var_decl) => {
                let value = if let Some(expr) = &var_decl.value {
                    let mut val = self.evaluate_expression(expr)?;
                    
                    // Convert arrays to lists if the type is List
                    if let crate::types::ChifType::List(_, _) = &var_decl.var_type {
                        if let ChifValue::Array(arr) = val {
                            val = ChifValue::List(arr);
                        }
                    }
                    
                    val
                } else {
                    ChifValue::Nil
                };
                
                self.set_variable(&var_decl.name, value)?;
            }
            Statement::Assignment(assignment) => {
                let value = self.evaluate_expression(&assignment.value)?;
                match &assignment.target {
                    Expression::Identifier(name) => {
                        self.set_variable(name, value)?;
                    }
                    Expression::Index(index_access) => {
                        self.assign_to_index(index_access, value)?;
                    }
                    Expression::FieldAccess(field_access) => {
                        self.assign_to_field(field_access, value)?;
                    }
                    _ => {
                        return Err(ChifError::RuntimeError {
                            message: "Invalid assignment target".to_string(),
                        });
                    }
                }
            }
            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
            }
            Statement::If(if_stmt) => {
                let condition = self.evaluate_expression(&if_stmt.condition)?;
                if self.is_truthy(&condition) {
                    self.execute_block(&if_stmt.then_block)?;
                } else if let Some(else_block) = &if_stmt.else_block {
                    self.execute_block(else_block)?;
                }
            }
            Statement::For(for_stmt) => {
                // Create new scope for the for loop variables
                self.locals.push(HashMap::new());
                
                // Execute initialization in the loop scope
                if let Some(init) = &for_stmt.init {
                    self.execute_statement(init)?;
                }
                
                // Save the current state of the loop variables after initialization
                let loop_scope_index = self.locals.len() - 1;
                
                loop {
                    if let Some(condition) = &for_stmt.condition {
                        let cond_value = self.evaluate_expression(condition)?;
                        if !self.is_truthy(&cond_value) {
                            break;
                        }
                    }
                    
                    // Execute the loop body
                    match self.execute_block(&for_stmt.body) {
                        Ok(()) => {},
                        Err(ChifError::Break) => break,
                        Err(ChifError::Continue) => {
                            // Execute update and continue
                            if let Some(update) = &for_stmt.update {
                                self.execute_statement(update)?;
                            }
                            continue;
                        },
                        Err(e) => return Err(e),
                    }
                    
                    if let Some(update) = &for_stmt.update {
                        // Execute update statement
                        self.execute_statement(update)?;
                    }
                    
                    // Preserve any changes to loop variables for the next iteration
                    // This ensures variables modified in the loop body remain modified
                    if loop_scope_index < self.locals.len() {
                        // We're still in the same scope structure
                        // No need to do anything special
                    } else {
                        // Something changed the scope structure, this is unexpected
                        // but we'll handle it gracefully
                        break;
                    }
                }
                
                // Сохраняем переменные из области видимости цикла в родительскую область
                if !self.locals.is_empty() {
                    let loop_scope = self.locals.last().unwrap().clone();
                    self.locals.pop();
                    
                    // Если есть родительская область видимости, копируем в неё измененные переменные
                    if !self.locals.is_empty() {
                        let parent_scope = self.locals.last_mut().unwrap();
                        
                        for (name, value) in loop_scope.iter() {
                            // Обновляем переменные в родительской области видимости
                            // Включая те, которые были объявлены до цикла
                            parent_scope.insert(name.clone(), value.clone());
                        }
                    }
                } else {
                    // На всякий случай, если список областей видимости пуст
                    // (не должно происходить, но для безопасности)
                    self.locals.push(HashMap::new());
                }
            }
            Statement::While(while_stmt) => {
                loop {
                    let condition = self.evaluate_expression(&while_stmt.condition)?;
                    if !self.is_truthy(&condition) {
                        break;
                    }
                    
                    match self.execute_block(&while_stmt.body) {
                        Ok(()) => {},
                        Err(ChifError::Break) => break,
                        Err(ChifError::Continue) => continue,
                        Err(e) => return Err(e),
                    }
                }
            }
            Statement::Switch(switch_stmt) => {
                let switch_value = self.evaluate_expression(&switch_stmt.expr)?;
                let mut matched = false;
                
                for case in &switch_stmt.cases {
                    let case_value = self.evaluate_expression(&case.value)?;
                    if self.values_equal(&switch_value, &case_value) {
                        self.execute_block(&case.body)?;
                        matched = true;
                        break;
                    }
                }
                
                if !matched {
                    if let Some(default_case) = &switch_stmt.default_case {
                        self.execute_block(default_case)?;
                    }
                }
            }
            Statement::Return(expr) => {
                let value = if let Some(expr) = expr {
                    self.evaluate_expression(expr)?
                } else {
                    ChifValue::Nil
                };
                
                return Err(ChifError::Return(value));
            }
            Statement::Break => {
                return Err(ChifError::Break);
            }
            Statement::Continue => {
                return Err(ChifError::Continue);
            }
        }
        Ok(())
    }
    
    fn evaluate_expression(&mut self, expr: &Expression) -> Result<ChifValue> {
        match expr {
            Expression::Literal(value) => {
                match value {
                    ChifValue::Str(s) => {
                        // Apply string interpolation to all string literals
                        let interpolated = self.interpolate_string(s)?;
                        Ok(ChifValue::Str(interpolated))
                    }
                    _ => Ok(value.clone()),
                }
            }
            Expression::Identifier(name) => {
                // Special built-in functions
                match name.as_str() {
                    "randi" => Ok(ChifValue::Str("randi".to_string())), // Placeholder
                    "randf" => Ok(ChifValue::Str("randf".to_string())), // Placeholder
                    "rands" => Ok(ChifValue::Str("rands".to_string())), // Placeholder
                    _ => self.get_variable(name),
                }
            }
            Expression::Binary(binary_op) => {
                let left = self.evaluate_expression(&binary_op.left)?;
                let right = self.evaluate_expression(&binary_op.right)?;
                self.apply_binary_op(&binary_op.operator, &left, &right)
            }
            Expression::Unary(unary_op) => {
                let operand = self.evaluate_expression(&unary_op.operand)?;
                self.apply_unary_op(&unary_op.operator, &operand)
            }
            Expression::Call(call) => {
                // Handle built-in functions
                match call.name.as_str() {
                    "randi" => {
                        if call.args.len() != 2 {
                            return Err(ChifError::RuntimeError {
                                message: "randi expects 2 arguments".to_string(),
                            });
                        }
                        let min = self.evaluate_expression(&call.args[0])?;
                        let max = self.evaluate_expression(&call.args[1])?;
                        
                        if let (ChifValue::Int(min_val), ChifValue::Int(max_val)) = (min, max) {
                            if min_val > max_val {
                                return Err(ChifError::RuntimeError {
                                    message: "randi: min cannot be greater than max".to_string(),
                                });
                            }
                            let mut rng = rand::thread_rng();
                            let result = rng.gen_range(min_val..=max_val);
                            Ok(ChifValue::Int(result))
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "randi expects integer arguments".to_string(),
                            })
                        }
                    }
                    "randf" => {
                        if call.args.len() != 2 {
                            return Err(ChifError::RuntimeError {
                                message: "randf expects 2 arguments".to_string(),
                            });
                        }
                        let min = self.evaluate_expression(&call.args[0])?;
                        let max = self.evaluate_expression(&call.args[1])?;
                        
                        if let (ChifValue::Float(min_val), ChifValue::Float(max_val)) = (min, max) {
                            if min_val > max_val {
                                return Err(ChifError::RuntimeError {
                                    message: "randf: min cannot be greater than max".to_string(),
                                });
                            }
                            let mut rng = rand::thread_rng();
                            let result = rng.gen_range(min_val..=max_val);
                            Ok(ChifValue::Float(result))
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "randf expects float arguments".to_string(),
                            })
                        }
                    }
                    "rands" => {
                        if call.args.len() != 2 {
                            return Err(ChifError::RuntimeError {
                                message: "rands expects 2 arguments".to_string(),
                            });
                        }
                        let from = self.evaluate_expression(&call.args[0])?;
                        let to = self.evaluate_expression(&call.args[1])?;
                        
                        if let (ChifValue::Str(from_str), ChifValue::Str(to_str)) = (from, to) {
                            if from_str.len() != 1 || to_str.len() != 1 {
                                return Err(ChifError::RuntimeError {
                                    message: "rands expects single character strings".to_string(),
                                });
                            }
                            let from_char = from_str.chars().next().unwrap() as u8;
                            let to_char = to_str.chars().next().unwrap() as u8;
                            
                            if from_char > to_char {
                                return Err(ChifError::RuntimeError {
                                    message: "rands: from cannot be greater than to".to_string(),
                                });
                            }
                            
                            let mut rng = rand::thread_rng();
                            let result_char = rng.gen_range(from_char..=to_char) as char;
                            Ok(ChifValue::Str(result_char.to_string()))
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "rands expects string arguments".to_string(),
                            })
                        }
                    }
                    "http_get" => {
                        if call.args.len() != 1 {
                            return Err(ChifError::RuntimeError {
                                message: "http_get expects 1 argument".to_string(),
                            });
                        }
                        let url = self.evaluate_expression(&call.args[0])?;
                        if let ChifValue::Str(url_str) = url {
                            self.http_get_request(&url_str)
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "http_get expects string URL".to_string(),
                            })
                        }
                    }
                    "http_post" => {
                        if call.args.len() != 2 {
                            return Err(ChifError::RuntimeError {
                                message: "http_post expects 2 arguments".to_string(),
                            });
                        }
                        let url = self.evaluate_expression(&call.args[0])?;
                        let body = self.evaluate_expression(&call.args[1])?;
                        if let (ChifValue::Str(url_str), ChifValue::Str(body_str)) = (url, body) {
                            self.http_post_request(&url_str, &body_str)
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "http_post expects string arguments".to_string(),
                            })
                        }
                    }
                    "http_put" => {
                        if call.args.len() != 2 {
                            return Err(ChifError::RuntimeError {
                                message: "http_put expects 2 arguments".to_string(),
                            });
                        }
                        let url = self.evaluate_expression(&call.args[0])?;
                        let body = self.evaluate_expression(&call.args[1])?;
                        if let (ChifValue::Str(url_str), ChifValue::Str(body_str)) = (url, body) {
                            self.http_put_request(&url_str, &body_str)
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "http_put expects string arguments".to_string(),
                            })
                        }
                    }
                    "http_delete" => {
                        if call.args.len() != 1 {
                            return Err(ChifError::RuntimeError {
                                message: "http_delete expects 1 argument".to_string(),
                            });
                        }
                        let url = self.evaluate_expression(&call.args[0])?;
                        if let ChifValue::Str(url_str) = url {
                            self.http_delete_request(&url_str)
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "http_delete expects string URL".to_string(),
                            })
                        }
                    }
                    _ => {
                        // Regular function call
                        let mut args = Vec::new();
                        for arg_expr in &call.args {
                            args.push(self.evaluate_expression(arg_expr)?);
                        }
                        
                        if let Some(func) = self.functions.get(&call.name).cloned() {
                            // Check if any arguments are references
                            let has_references = call.args.iter().any(|arg| {
                                matches!(arg, Expression::Reference(_))
                            });
                            
                            if has_references {
                                self.call_function_with_references(&func, args, &call.args)
                            } else {
                                self.call_function(&func, args)
                            }
                        } else {
                            Err(ChifError::FunctionNotFound {
                                name: call.name.clone(),
                            })
                        }
                    }
                }
            }
            Expression::MethodCall(method_call) => {
                // Special handling for module function calls (module.function())
                if let Expression::Identifier(module_name) = &*method_call.object {
                    // Check if this is a module call
                    if let Some(module) = self.modules.get(module_name) {
                        if let Some(func) = module.functions.get(&method_call.method).cloned() {
                            let mut args = Vec::new();
                            for arg_expr in &method_call.args {
                                args.push(self.evaluate_expression(arg_expr)?);
                            }
                            return self.call_function(&func, args);
                        } else {
                            return Err(ChifError::FunctionNotFound {
                                name: format!("{}.{}", module_name, method_call.method),
                            });
                        }
                    }
                    
                    // Special handling for mutable methods on variables
                    if method_call.method == "add" || method_call.method == "addAt" || method_call.method == "del" {
                        return self.call_mutable_method(module_name, &method_call.method, &method_call.args);
                    }
                    
                    // Check if this is a struct method that might mutate self
                    let object = self.get_variable(module_name)?;
                    if let ChifValue::Struct(struct_name, _) = &object {
                        if let Some(methods) = self.struct_methods.get(struct_name).cloned() {
                            for method in &methods {
                                if method.name == method_call.method {
                                    return self.call_mutable_struct_method(module_name, &method_call.method, &method_call.args);
                                }
                            }
                        }
                    }
                }
                
                let object = self.evaluate_expression(&method_call.object)?;
                self.call_method(&object, &method_call.method, &method_call.args)
            }
            Expression::Index(index_access) => {
                let object = self.evaluate_expression(&index_access.object)?;
                let mut current = object;
                
                for index_expr in &index_access.indices {
                    let index = self.evaluate_expression(index_expr)?;
                    current = self.get_index(&current, &index)?;
                }
                
                Ok(current)
            }
            Expression::FieldAccess(field_access) => {
                let object = self.evaluate_expression(&field_access.object)?;
                self.get_field(&object, &field_access.field)
            }
            Expression::ArrayLiteral(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_expression(element)?);
                }
                // Check if this should be a list or array based on context
                // For now, we'll create arrays by default
                Ok(ChifValue::Array(values))
            }
            Expression::MapLiteral(pairs) => {
                let mut map = HashMap::new();
                for (key_expr, value_expr) in pairs {
                    let key = self.evaluate_expression(key_expr)?;
                    let value = self.evaluate_expression(value_expr)?;
                    
                    if let ChifValue::Str(key_str) = key {
                        map.insert(key_str, value);
                    } else {
                        return Err(ChifError::RuntimeError {
                            message: "Map keys must be strings".to_string(),
                        });
                    }
                }
                Ok(ChifValue::Map(map))
            }
            Expression::StructLiteral(struct_literal) => {
                let mut fields = HashMap::new();
                for (field_name, field_expr) in &struct_literal.fields {
                    let field_value = self.evaluate_expression(field_expr)?;
                    fields.insert(field_name.clone(), field_value);
                }
                Ok(ChifValue::Struct(struct_literal.struct_name.clone(), fields))
            }
            Expression::Reference(expr) => {
                // Create a reference to a variable
                if let Expression::Identifier(var_name) = &**expr {
                    Ok(ChifValue::Reference(var_name.clone()))
                } else {
                    // For complex expressions, create a pointer to the value
                    let value = self.evaluate_expression(expr)?;
                    Ok(ChifValue::Pointer(Box::new(value)))
                }
            }
            Expression::Dereference(expr) => {
                let value = self.evaluate_expression(expr)?;
                match value {
                    ChifValue::Pointer(inner) => Ok(*inner),
                    ChifValue::Reference(var_name) => {
                        // Dereference a variable reference
                        self.get_variable(&var_name)
                    }
                    _ => Err(ChifError::RuntimeError {
                        message: "Cannot dereference non-pointer value".to_string(),
                    })
                }
            }
        }
    }
    
    fn call_method(&mut self, object: &ChifValue, method_name: &str, args: &[Expression]) -> Result<ChifValue> {
        match object {
            ChifValue::Array(_) => {
                match method_name {
                    "len" => {
                        if let ChifValue::Array(arr) = object {
                            Ok(ChifValue::Int(arr.len() as i64))
                        } else {
                            unreachable!()
                        }
                    }
                    _ => Err(ChifError::RuntimeError {
                        message: format!("Method '{}' not supported for arrays (immutable)", method_name),
                    }),
                }
            }
            ChifValue::List(_) => {
                match method_name {
                    "len" => {
                        if let ChifValue::List(list) = object {
                            Ok(ChifValue::Int(list.len() as i64))
                        } else {
                            unreachable!()
                        }
                    }
                    "add" => {
                        if args.len() != 1 {
                            return Err(ChifError::RuntimeError {
                                message: "add method expects 1 argument".to_string(),
                            });
                        }
                        // Note: This is still a simplified implementation
                        // In a real implementation, we'd need mutable references
                        Ok(ChifValue::Nil)
                    }
                    "addAt" => {
                        if args.len() != 2 {
                            return Err(ChifError::RuntimeError {
                                message: "addAt method expects 2 arguments".to_string(),
                            });
                        }
                        // Note: This is still a simplified implementation
                        Ok(ChifValue::Nil)
                    }
                    _ => Err(ChifError::RuntimeError {
                        message: format!("Unknown method '{}' for list", method_name),
                    }),
                }
            }
            ChifValue::Str(s) => {
                match method_name {
                    "len" => Ok(ChifValue::Int(s.len() as i64)),
                    _ => Err(ChifError::RuntimeError {
                        message: format!("Unknown method '{}' for string", method_name),
                    }),
                }
            }
            ChifValue::Struct(struct_name, _) if struct_name == "Console" => {
                // Handle console methods
                if method_name == "out" && args.len() == 1 {
                    let arg = self.evaluate_expression(&args[0])?;
                    let output = self.format_output(&arg)?;
                    println!("{}", output);
                    Ok(ChifValue::Nil)
                } else if method_name == "in" && args.len() == 1 {
                    // Handle console input with pointer
                    if let Expression::Dereference(ref inner) = &args[0] {
                        if let Expression::Identifier(var_name) = &**inner {
                            let mut input = String::new();
                            io::stdin().read_line(&mut input).unwrap();
                            let input = input.trim().to_string();
                            
                            // Update the variable
                            self.set_variable(var_name, ChifValue::Str(input))?;
                            Ok(ChifValue::Nil)
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "con.in expects a pointer to a variable".to_string(),
                            })
                        }
                    } else {
                        Err(ChifError::RuntimeError {
                            message: "con.in expects a dereferenced variable (*var)".to_string(),
                        })
                    }
                } else {
                    Err(ChifError::RuntimeError {
                        message: format!("Unknown console method '{}'", method_name),
                    })
                }
            }
            ChifValue::Struct(struct_name, _) => {
                // Проверяем, является ли вызов метода на переменной
                if let Expression::MethodCall(method_call) = args[0].clone() {
                    if let Expression::Identifier(var_name) = *method_call.object {
                        // Используем call_mutable_struct_method для вызова метода на переменной
                        return self.call_mutable_struct_method(&var_name, method_name, &args[1..]);
                    }
                }
                
                // Handle struct methods
                let methods = self.struct_methods.get(struct_name).cloned();
                if let Some(methods) = methods {
                    for method in methods {
                        if method.name == method_name {
                            let mut method_args = vec![object.clone()]; // self parameter
                            for arg_expr in args {
                                method_args.push(self.evaluate_expression(arg_expr)?);
                            }
                            return self.call_function(&method, method_args);
                        }
                    }
                }
                Err(ChifError::RuntimeError {
                    message: format!("Unknown method '{}' for struct '{}'", method_name, struct_name),
                })
            }
            _ => {
                Err(ChifError::RuntimeError {
                    message: format!("Unknown method '{}'", method_name),
                })
            }
        }
    }
    
    fn format_output(&mut self, value: &ChifValue) -> Result<String> {
        match value {
            ChifValue::Str(s) => {
                // Handle string interpolation
                self.interpolate_string(s)
            }
            _ => Ok(value.to_string()),
        }
    }
    
    fn interpolate_string(&mut self, s: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Check if this is an escaped brace
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume second '{'
                    result.push('{');
                    continue;
                }
                
                // Parse variable name
                let mut var_name = String::new();
                let mut found_closing = false;
                
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        found_closing = true;
                        break;
                    }
                    var_name.push(ch);
                }
                
                if !found_closing {
                    return Err(ChifError::RuntimeError {
                        message: "Unclosed interpolation bracket '{'".to_string(),
                    });
                }
                
                if var_name.is_empty() {
                    result.push_str("{}");
                } else {
                    // Evaluate the complex expression
                    match self.evaluate_interpolation_expression(&var_name) {
                        Ok(value) => {
                            result.push_str(&value.to_string());
                        }
                        Err(_) => {
                            // If expression evaluation failed, keep the placeholder
                            result.push_str(&format!("{{{}}}", var_name));
                        }
                    }
                }
            } else if ch == '}' && chars.peek() == Some(&'}') {
                // Escaped closing brace
                chars.next(); // consume second '}'
                result.push('}');
            } else {
                result.push(ch);
            }
        }
        
        Ok(result)
    }
    
    // New helper method to evaluate complex interpolation expressions
    fn evaluate_interpolation_expression(&mut self, expr: &str) -> Result<ChifValue> {
        // Handle method calls like names.len()
        if expr.contains('.') && expr.ends_with("()") {
            let method_call = &expr[..expr.len()-2]; // Remove ()
            let parts: Vec<&str> = method_call.split('.').collect();
            if parts.len() == 2 {
                let obj = self.get_variable(parts[0])?;
                return self.call_method(&obj, parts[1], &[]);
            }
            return Err(ChifError::RuntimeError {
                message: format!("Invalid method call expression: {}", expr),
            });
        }
        
        // Handle combined indexing and field access like users[0].name
        if expr.contains('[') && expr.contains(']') && expr.contains('.') {
            // First, get the base variable
            let base_end = expr.find('[').unwrap_or(expr.len());
            let base_name = &expr[..base_end];
            let mut current_value = self.get_variable(base_name)?;
            
            // Parse the rest of the expression
            let mut pos = base_end;
            let chars: Vec<char> = expr.chars().collect();
            
            while pos < chars.len() {
                if chars[pos] == '[' {
                    // Handle array indexing
                    let start_idx = pos + 1;
                    let mut end_idx = start_idx;
                    while end_idx < chars.len() && chars[end_idx] != ']' {
                        end_idx += 1;
                    }
                    
                    if end_idx >= chars.len() {
                        return Err(ChifError::RuntimeError {
                            message: format!("Unclosed bracket in expression: {}", expr),
                        });
                    }
                    
                    let index_str = &expr[start_idx..end_idx];
                    let index = index_str.parse::<i64>().map_err(|_| ChifError::RuntimeError {
                        message: format!("Invalid array index: {}", index_str),
                    })?;
                    
                    current_value = self.get_index(&current_value, &ChifValue::Int(index))?;
                    pos = end_idx + 1;
                } else if chars[pos] == '.' {
                    // Handle field access
                    let start_field = pos + 1;
                    let mut end_field = start_field;
                    while end_field < chars.len() && chars[end_field] != '.' && chars[end_field] != '[' {
                        end_field += 1;
                    }
                    
                    let field_name = &expr[start_field..end_field];
                    current_value = self.get_field(&current_value, field_name)?;
                    pos = end_field;
                } else {
                    pos += 1;
                }
            }
            
            return Ok(current_value);
        }
        
        // Handle simple field access like person.name
        if expr.contains('.') {
            let parts: Vec<&str> = expr.split('.').collect();
            if parts.len() >= 2 {
                let mut obj = self.get_variable(parts[0])?;
                
                // Navigate through the chain of fields
                for field_name in &parts[1..] {
                    obj = self.get_field(&obj, field_name)?;
                }
                
                return Ok(obj);
            }
        }
        
        // Handle simple indexing like numbers[0]
        if expr.contains('[') && expr.contains(']') {
            if let Some(bracket_pos) = expr.find('[') {
                let var_part = &expr[..bracket_pos];
                let index_part = &expr[bracket_pos+1..];
                if let Some(close_pos) = index_part.find(']') {
                    let index_str = &index_part[..close_pos];
                    let index = index_str.parse::<i64>().map_err(|_| ChifError::RuntimeError {
                        message: format!("Invalid array index: {}", index_str),
                    })?;
                    
                    let obj = self.get_variable(var_part)?;
                    return self.get_index(&obj, &ChifValue::Int(index));
                }
            }
        }
        
        // Simple variable lookup
        self.get_variable(expr)
    }
    
    fn apply_binary_op(&self, op: &BinaryOperator, left: &ChifValue, right: &ChifValue) -> Result<ChifValue> {
        match (left, right) {
            (ChifValue::Int(l), ChifValue::Int(r)) => {
                match op {
                    BinaryOperator::Add => Ok(ChifValue::Int(l + r)),
                    BinaryOperator::Subtract => Ok(ChifValue::Int(l - r)),
                    BinaryOperator::Multiply => Ok(ChifValue::Int(l * r)),
                    BinaryOperator::Divide => {
                        if *r == 0 {
                            Err(ChifError::RuntimeError {
                                message: "Division by zero".to_string(),
                            })
                        } else {
                            Ok(ChifValue::Int(l / r))
                        }
                    }
                    BinaryOperator::Modulo => Ok(ChifValue::Int(l % r)),
                    BinaryOperator::Equal => Ok(ChifValue::Bool(l == r)),
                    BinaryOperator::NotEqual => Ok(ChifValue::Bool(l != r)),
                    BinaryOperator::Less => Ok(ChifValue::Bool(l < r)),
                    BinaryOperator::Greater => Ok(ChifValue::Bool(l > r)),
                    BinaryOperator::LessEqual => Ok(ChifValue::Bool(l <= r)),
                    BinaryOperator::GreaterEqual => Ok(ChifValue::Bool(l >= r)),
                    _ => Err(ChifError::RuntimeError {
                        message: format!("Invalid operation for integers: {:?}", op),
                    }),
                }
            }
            (ChifValue::Float(l), ChifValue::Float(r)) => {
                match op {
                    BinaryOperator::Add => Ok(ChifValue::Float(l + r)),
                    BinaryOperator::Subtract => Ok(ChifValue::Float(l - r)),
                    BinaryOperator::Multiply => Ok(ChifValue::Float(l * r)),
                    BinaryOperator::Divide => Ok(ChifValue::Float(l / r)),
                    BinaryOperator::Equal => Ok(ChifValue::Bool((l - r).abs() < f64::EPSILON)),
                    BinaryOperator::NotEqual => Ok(ChifValue::Bool((l - r).abs() >= f64::EPSILON)),
                    BinaryOperator::Less => Ok(ChifValue::Bool(l < r)),
                    BinaryOperator::Greater => Ok(ChifValue::Bool(l > r)),
                    BinaryOperator::LessEqual => Ok(ChifValue::Bool(l <= r)),
                    BinaryOperator::GreaterEqual => Ok(ChifValue::Bool(l >= r)),
                    _ => Err(ChifError::RuntimeError {
                        message: format!("Invalid operation for floats: {:?}", op),
                    }),
                }
            }
            (ChifValue::Str(l), ChifValue::Str(r)) => {
                match op {
                    BinaryOperator::Add => Ok(ChifValue::Str(format!("{}{}", l, r))),
                    BinaryOperator::Equal => Ok(ChifValue::Bool(l == r)),
                    BinaryOperator::NotEqual => Ok(ChifValue::Bool(l != r)),
                    BinaryOperator::Less => Ok(ChifValue::Bool(l < r)),
                    BinaryOperator::Greater => Ok(ChifValue::Bool(l > r)),
                    BinaryOperator::LessEqual => Ok(ChifValue::Bool(l <= r)),
                    BinaryOperator::GreaterEqual => Ok(ChifValue::Bool(l >= r)),
                    _ => Err(ChifError::RuntimeError {
                        message: format!("Invalid operation for strings: {:?}", op),
                    }),
                }
            }
            (ChifValue::Bool(l), ChifValue::Bool(r)) => {
                match op {
                    BinaryOperator::And => Ok(ChifValue::Bool(*l && *r)),
                    BinaryOperator::Or => Ok(ChifValue::Bool(*l || *r)),
                    BinaryOperator::Equal => Ok(ChifValue::Bool(l == r)),
                    BinaryOperator::NotEqual => Ok(ChifValue::Bool(l != r)),
                    _ => Err(ChifError::RuntimeError {
                        message: format!("Invalid operation for booleans: {:?}", op),
                    }),
                }
            }
            _ => Err(ChifError::RuntimeError {
                message: format!("Type mismatch in binary operation: {:?} {:?} {:?}", left, op, right),
            }),
        }
    }
    
    fn apply_unary_op(&self, op: &UnaryOperator, operand: &ChifValue) -> Result<ChifValue> {
        match (op, operand) {
            (UnaryOperator::Not, ChifValue::Bool(b)) => Ok(ChifValue::Bool(!b)),
            (UnaryOperator::Minus, ChifValue::Int(i)) => Ok(ChifValue::Int(-i)),
            (UnaryOperator::Minus, ChifValue::Float(f)) => Ok(ChifValue::Float(-f)),
            _ => Err(ChifError::RuntimeError {
                message: format!("Invalid unary operation: {:?} {:?}", op, operand),
            }),
        }
    }
    
    fn get_variable(&self, name: &str) -> Result<ChifValue> {
        // Check locals first (from innermost to outermost)
        for scope in self.locals.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        
        // Check globals
        if let Some(value) = self.globals.get(name) {
            Ok(value.clone())
        } else {
            Err(ChifError::VariableNotFound {
                name: name.to_string(),
            })
        }
    }
    
    fn set_variable(&mut self, name: &str, value: ChifValue) -> Result<()> {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), value);
        } else {
            self.globals.insert(name.to_string(), value);
        }
        Ok(())
    }
    
    fn get_index(&self, object: &ChifValue, index: &ChifValue) -> Result<ChifValue> {
        match (object, index) {
            (ChifValue::Array(arr), ChifValue::Int(i)) => {
                let idx = *i as usize;
                if idx < arr.len() {
                    Ok(arr[idx].clone())
                } else {
                    Err(ChifError::IndexOutOfBounds { index: idx })
                }
            }
            (ChifValue::List(list), ChifValue::Int(i)) => {
                let idx = *i as usize;
                if idx < list.len() {
                    Ok(list[idx].clone())
                } else {
                    Err(ChifError::IndexOutOfBounds { index: idx })
                }
            }
            (ChifValue::Map(map), ChifValue::Str(key)) => {
                if let Some(value) = map.get(key) {
                    Ok(value.clone())
                } else {
                    Ok(ChifValue::Nil)
                }
            }
            _ => Err(ChifError::RuntimeError {
                message: "Invalid index operation".to_string(),
            }),
        }
    }
    
    fn get_field(&self, object: &ChifValue, field: &str) -> Result<ChifValue> {
        match object {
            ChifValue::Struct(_, fields) => {
                if let Some(value) = fields.get(field) {
                    Ok(value.clone())
                } else {
                    Err(ChifError::RuntimeError {
                        message: format!("Field '{}' not found", field),
                    })
                }
            }
            _ => Err(ChifError::RuntimeError {
                message: "Cannot access field on non-struct value".to_string(),
            }),
        }
    }
    
    fn assign_to_index(&mut self, _index_access: &IndexAccess, _value: ChifValue) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, we'd need to handle mutable references properly
        Ok(())
    }
    
    fn assign_to_field(&mut self, _field_access: &FieldAccess, _value: ChifValue) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, we'd need to handle mutable references properly
        Ok(())
    }
    
    fn is_truthy(&self, value: &ChifValue) -> bool {
        match value {
            ChifValue::Bool(b) => *b,
            ChifValue::Nil => false,
            ChifValue::Int(i) => *i != 0,
            ChifValue::Float(f) => *f != 0.0,
            ChifValue::Str(s) => !s.is_empty(),
            _ => true,
        }
    }
    
    fn process_import(&mut self, import: &ImportStatement) -> Result<()> {
        use std::fs;
        use crate::{lexer::Lexer, parser::Parser};
        
        // Add .rono extension if not present
        let file_path = if import.path.ends_with(".rono") {
            import.path.clone()
        } else {
            format!("{}.rono", import.path)
        };
        
        // Read the imported file
        let source = fs::read_to_string(&file_path).map_err(|_| {
            ChifError::RuntimeError {
                message: format!("Cannot read file: {}", file_path),
            }
        })?;
        
        // Parse the imported file
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let imported_program = parser.parse()?;
        
        // Extract functions and structs from imported module
        let mut module_functions = HashMap::new();
        let mut module_structs = HashMap::new();
        
        for item in &imported_program.items {
            match item {
                Item::Function(func) => {
                    module_functions.insert(func.name.clone(), func.clone());
                    // Also add to global functions for recursive calls
                    self.functions.insert(func.name.clone(), func.clone());
                }
                Item::Struct(struct_def) => {
                    module_structs.insert(struct_def.name.clone(), struct_def.clone());
                    // Also add to global structs so they can be used
                    self.structs.insert(struct_def.name.clone(), struct_def.clone());
                }
                Item::StructImpl(impl_block) => {
                    // Add struct methods to global struct_methods
                    self.struct_methods
                        .entry(impl_block.struct_name.clone())
                        .or_insert_with(Vec::new)
                        .extend(impl_block.methods.clone());
                }
                _ => {} // Ignore nested imports for now
            }
        }
        
        let module = Module {
            functions: module_functions,
            structs: module_structs,
        };
        
        // Store module with alias or filename
        let module_name = import.alias.clone().unwrap_or_else(|| {
            // Extract filename without extension
            std::path::Path::new(&import.path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string()
        });
        
        self.modules.insert(module_name, module);
        Ok(())
    }
    
    fn http_get_request(&self, url: &str) -> Result<ChifValue> {
        use reqwest::blocking::Client;
        use std::collections::HashMap;
        
        let client = Client::new();
        match client.get(url).send() {
            Ok(response) => {
                let status = response.status().as_u16() as i64;
                let body = response.text().unwrap_or_else(|_| "Error reading response".to_string());
                
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), ChifValue::Int(status));
                fields.insert("body".to_string(), ChifValue::Str(body));
                fields.insert("content_type".to_string(), ChifValue::Str("application/json".to_string()));
                
                Ok(ChifValue::Struct("HttpResponse".to_string(), fields))
            }
            Err(e) => {
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), ChifValue::Int(0));
                fields.insert("body".to_string(), ChifValue::Str(format!("Request failed: {}", e)));
                fields.insert("content_type".to_string(), ChifValue::Str("text/plain".to_string()));
                
                Ok(ChifValue::Struct("HttpResponse".to_string(), fields))
            }
        }
    }
    
    fn http_post_request(&self, url: &str, body: &str) -> Result<ChifValue> {
        use reqwest::blocking::Client;
        use std::collections::HashMap;
        
        let client = Client::new();
        match client.post(url).body(body.to_string()).header("Content-Type", "application/json").send() {
            Ok(response) => {
                let status = response.status().as_u16() as i64;
                let response_body = response.text().unwrap_or_else(|_| "Error reading response".to_string());
                
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), ChifValue::Int(status));
                fields.insert("body".to_string(), ChifValue::Str(response_body));
                fields.insert("content_type".to_string(), ChifValue::Str("application/json".to_string()));
                
                Ok(ChifValue::Struct("HttpResponse".to_string(), fields))
            }
            Err(e) => {
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), ChifValue::Int(0));
                fields.insert("body".to_string(), ChifValue::Str(format!("Request failed: {}", e)));
                fields.insert("content_type".to_string(), ChifValue::Str("text/plain".to_string()));
                
                Ok(ChifValue::Struct("HttpResponse".to_string(), fields))
            }
        }
    }
    
    fn http_put_request(&self, url: &str, body: &str) -> Result<ChifValue> {
        use reqwest::blocking::Client;
        use std::collections::HashMap;
        
        let client = Client::new();
        match client.put(url).body(body.to_string()).header("Content-Type", "application/json").send() {
            Ok(response) => {
                let status = response.status().as_u16() as i64;
                let response_body = response.text().unwrap_or_else(|_| "Error reading response".to_string());
                
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), ChifValue::Int(status));
                fields.insert("body".to_string(), ChifValue::Str(response_body));
                fields.insert("content_type".to_string(), ChifValue::Str("application/json".to_string()));
                
                Ok(ChifValue::Struct("HttpResponse".to_string(), fields))
            }
            Err(e) => {
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), ChifValue::Int(0));
                fields.insert("body".to_string(), ChifValue::Str(format!("Request failed: {}", e)));
                fields.insert("content_type".to_string(), ChifValue::Str("text/plain".to_string()));
                
                Ok(ChifValue::Struct("HttpResponse".to_string(), fields))
            }
        }
    }
    
    fn http_delete_request(&self, url: &str) -> Result<ChifValue> {
        use reqwest::blocking::Client;
        use std::collections::HashMap;
        
        let client = Client::new();
        match client.delete(url).send() {
            Ok(response) => {
                let status = response.status().as_u16() as i64;
                let response_body = response.text().unwrap_or_else(|_| "Error reading response".to_string());
                
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), ChifValue::Int(status));
                fields.insert("body".to_string(), ChifValue::Str(response_body));
                fields.insert("content_type".to_string(), ChifValue::Str("text/plain".to_string()));
                
                Ok(ChifValue::Struct("HttpResponse".to_string(), fields))
            }
            Err(e) => {
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), ChifValue::Int(0));
                fields.insert("body".to_string(), ChifValue::Str(format!("Request failed: {}", e)));
                fields.insert("content_type".to_string(), ChifValue::Str("text/plain".to_string()));
                
                Ok(ChifValue::Struct("HttpResponse".to_string(), fields))
            }
        }
    }
    
    fn values_equal(&self, left: &ChifValue, right: &ChifValue) -> bool {
        match (left, right) {
            (ChifValue::Int(l), ChifValue::Int(r)) => l == r,
            (ChifValue::Float(l), ChifValue::Float(r)) => (l - r).abs() < f64::EPSILON,
            (ChifValue::Str(l), ChifValue::Str(r)) => l == r,
            (ChifValue::Bool(l), ChifValue::Bool(r)) => l == r,
            (ChifValue::Nil, ChifValue::Nil) => true,
            _ => false,
        }
    }
    
    fn call_function_with_references(&mut self, func: &Function, args: Vec<ChifValue>, arg_exprs: &[Expression]) -> Result<ChifValue> {
        if args.len() != func.params.len() {
            return Err(ChifError::RuntimeError {
                message: format!(
                    "Function '{}' expects {} arguments, got {}",
                    func.name,
                    func.params.len(),
                    args.len()
                ),
            });
        }
        
        // Track which parameters are references to variables
        let mut var_refs = Vec::new();
        for (i, arg_expr) in arg_exprs.iter().enumerate() {
            if let Expression::Reference(ref inner) = arg_expr {
                if let Expression::Identifier(var_name) = &**inner {
                    var_refs.push((i, var_name.clone()));
                }
            }
        }
        
        // Create new scope
        let mut scope = HashMap::new();
        
        // Bind parameters
        for (param, arg) in func.params.iter().zip(args.iter()) {
            scope.insert(param.name.clone(), arg.clone());
        }
        
        self.locals.push(scope);
        
        let result = self.execute_block(&func.body);
        
        // Update referenced variables after function execution
        let updates: Vec<(String, ChifValue)> = if let Some(local_scope) = self.locals.last() {
            var_refs.iter().filter_map(|(param_idx, var_name)| {
                func.params.get(*param_idx).and_then(|param| {
                    local_scope.get(&param.name).map(|updated_value| {
                        (var_name.clone(), updated_value.clone())
                    })
                })
            }).collect()
        } else {
            Vec::new()
        };
        
        self.locals.pop();
        
        // Apply updates after popping the scope
        for (var_name, updated_value) in updates {
            self.set_variable(&var_name, updated_value)?;
        }
        
        match result {
            Ok(_) => Ok(ChifValue::Nil),
            Err(ChifError::Return(value)) => Ok(value),
            Err(e) => Err(e),
        }
    }
    
    fn call_mutable_struct_method(&mut self, var_name: &str, method_name: &str, args: &[Expression]) -> Result<ChifValue> {
        // Получаем объект
        let object = self.get_variable(var_name)?;
        
        if let ChifValue::Struct(struct_name, fields) = &object {
            let struct_name = struct_name.clone();
            let fields_clone = fields.clone();
            let methods = self.struct_methods.get(&struct_name).cloned();
            
            if let Some(methods) = methods {
                for method in methods {
                    if method.name == method_name {
                        // Создаем аргументы для вызова функции
                        let mut method_args = vec![object.clone()]; // self parameter
                        
                        // Получаем аргументы
                        let mut dx_val = 0;
                        let mut dy_val = 0;
                        
                        if method_name == "shift" && args.len() >= 2 {
                            let dx = self.evaluate_expression(&args[0])?;
                            let dy = self.evaluate_expression(&args[1])?;
                            
                            if let (ChifValue::Int(dx), ChifValue::Int(dy)) = (dx, dy) {
                                dx_val = dx;
                                dy_val = dy;
                            }
                        }
                        
                        for arg_expr in args {
                            method_args.push(self.evaluate_expression(arg_expr)?);
                        }
                        
                        // Вызываем функцию
                        let result = self.call_function(&method, method_args)?;
                        
                        // Обновляем поля структуры после вызова метода
                        if method_name == "shift" {
                            let mut updated_fields = fields_clone.clone();
                            
                            // Обновляем поля x и y
                            if let Some(ChifValue::Int(x)) = updated_fields.get("x") {
                                updated_fields.insert("x".to_string(), ChifValue::Int(x + dx_val));
                            }
                            if let Some(ChifValue::Int(y)) = updated_fields.get("y") {
                                updated_fields.insert("y".to_string(), ChifValue::Int(y + dy_val));
                            }
                            
                            // Создаем обновленную структуру
                            let updated_object = ChifValue::Struct(struct_name, updated_fields);
                            
                            // Обновляем объект в переменной
                            self.set_variable(var_name, updated_object)?;
                        }
                        
                        return Ok(result);
                    }
                }
            }
        }
        
        Err(ChifError::RuntimeError {
            message: format!("Method '{}' not found for struct", method_name),
        })
    }
    
    fn call_mutable_method(&mut self, var_name: &str, method_name: &str, args: &[Expression]) -> Result<ChifValue> {
        let mut object = self.get_variable(var_name)?;
        
        match &mut object {
            ChifValue::List(list) => {
                match method_name {
                    "add" => {
                        if args.len() != 1 {
                            return Err(ChifError::RuntimeError {
                                message: "add method expects 1 argument".to_string(),
                            });
                        }
                        let value = self.evaluate_expression(&args[0])?;
                        list.push(value);
                        self.set_variable(var_name, object)?;
                        Ok(ChifValue::Nil)
                    }
                    "addAt" => {
                        if args.len() != 2 {
                            return Err(ChifError::RuntimeError {
                                message: "addAt method expects 2 arguments".to_string(),
                            });
                        }
                        let value = self.evaluate_expression(&args[0])?;
                        let index = self.evaluate_expression(&args[1])?;
                        
                        if let ChifValue::Int(idx) = index {
                            if idx >= 0 && (idx as usize) <= list.len() {
                                list.insert(idx as usize, value);
                                self.set_variable(var_name, object)?;
                                Ok(ChifValue::Nil)
                            } else {
                                Err(ChifError::RuntimeError {
                                    message: format!("Index {} out of bounds for list of length {}", idx, list.len()),
                                })
                            }
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "addAt index must be an integer".to_string(),
                            })
                        }
                    }
                    "del" => {
                        if args.len() != 1 {
                            return Err(ChifError::RuntimeError {
                                message: "del method expects 1 argument".to_string(),
                            });
                        }
                        let index = self.evaluate_expression(&args[0])?;
                        
                        if let ChifValue::Int(idx) = index {
                            if idx >= 0 && (idx as usize) < list.len() {
                                list.remove(idx as usize);
                                self.set_variable(var_name, object)?;
                                Ok(ChifValue::Nil)
                            } else {
                                Err(ChifError::RuntimeError {
                                    message: format!("Index {} out of bounds for list of length {}", idx, list.len()),
                                })
                            }
                        } else {
                            Err(ChifError::RuntimeError {
                                message: "del index must be an integer".to_string(),
                            })
                        }
                    }
                    _ => Err(ChifError::RuntimeError {
                        message: format!("Unknown mutable method '{}' for list", method_name),
                    }),
                }
            }
            _ => Err(ChifError::RuntimeError {
                message: format!("Method '{}' not supported for this type", method_name),
            }),
        }
    }
    

}