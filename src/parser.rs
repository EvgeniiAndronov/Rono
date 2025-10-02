use crate::ast::*;
use crate::error::{ChifError, Result};
use crate::lexer::Token;
use crate::types::{ChifType, ChifValue};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<Program> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }
        
        Ok(Program { items })
    }
    
    fn parse_item(&mut self) -> Result<Item> {
        match &self.peek() {
            Token::Import => {
                let import = self.parse_import()?;
                Ok(Item::Import(import))
            }
            Token::Chif => {
                self.advance(); // consume 'chif'
                let func = self.parse_function(true)?;
                Ok(Item::Function(func))
            }
            Token::Fn => {
                let func = self.parse_function(false)?;
                Ok(Item::Function(func))
            }
            Token::FnFor => {
                let impl_block = self.parse_struct_impl()?;
                Ok(Item::StructImpl(impl_block))
            }
            Token::Struct => {
                let struct_def = self.parse_struct_def()?;
                Ok(Item::Struct(struct_def))
            }
            _ => Err(ChifError::ParserError {
                message: format!("Expected import, function, struct, or struct implementation, found {:?}", self.peek()),
            }),
        }
    }
    
    fn parse_import(&mut self) -> Result<ImportStatement> {
        self.consume(Token::Import, "Expected 'import'")?;
        
        let path = match self.advance() {
            Token::StringLiteral(path) => path,
            _ => return Err(ChifError::ParserError {
                message: "Expected string literal after 'import'".to_string(),
            }),
        };
        
        let alias = if self.match_token(&Token::As) {
            match self.advance() {
                Token::Identifier(alias) => Some(alias),
                _ => return Err(ChifError::ParserError {
                    message: "Expected identifier after 'as'".to_string(),
                }),
            }
        } else {
            None
        };
        
        self.consume(Token::Semicolon, "Expected ';' after import statement")?;
        
        Ok(ImportStatement { path, alias })
    }
    
    fn parse_function(&mut self, is_main: bool) -> Result<Function> {
        if !is_main {
            self.consume(Token::Fn, "Expected 'fn'")?;
        }
        
        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => return Err(ChifError::ParserError {
                message: "Expected function name".to_string(),
            }),
        };
        
        self.consume(Token::LeftParen, "Expected '(' after function name")?;
        
        let mut params = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                let param_name = match self.advance() {
                    Token::Identifier(name) => name,
                    _ => return Err(ChifError::ParserError {
                        message: "Expected parameter name".to_string(),
                    }),
                };
                
                // Special handling for 'self' parameter
                let param_type = if param_name == "self" {
                    ChifType::Struct("Self".to_string()) // Special type for self
                } else {
                    self.consume(Token::Colon, "Expected ':' after parameter name")?;
                    self.parse_type()?
                };
                
                params.push(Parameter {
                    name: param_name,
                    param_type,
                });
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(Token::RightParen, "Expected ')' after parameters")?;
        
        let return_type = if !self.check(&Token::LeftBrace) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        Ok(Function {
            name,
            params,
            return_type,
            body,
            is_main,
        })
    }
    
    fn parse_struct_def(&mut self) -> Result<StructDef> {
        self.consume(Token::Struct, "Expected 'struct'")?;
        
        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => return Err(ChifError::ParserError {
                message: "Expected struct name".to_string(),
            }),
        };
        
        self.consume(Token::LeftBrace, "Expected '{' after struct name")?;
        
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let field_name = match self.advance() {
                Token::Identifier(name) => name,
                _ => return Err(ChifError::ParserError {
                    message: "Expected field name".to_string(),
                }),
            };
            
            self.consume(Token::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;
            self.consume(Token::Comma, "Expected ',' after field type")?;
            
            fields.push(StructField {
                name: field_name,
                field_type,
            });
        }
        
        self.consume(Token::RightBrace, "Expected '}' after struct fields")?;
        
        Ok(StructDef { name, fields })
    }
    
    fn parse_struct_impl(&mut self) -> Result<StructImpl> {
        self.consume(Token::FnFor, "Expected 'fn_for'")?;
        
        let struct_name = match self.advance() {
            Token::Identifier(name) => name,
            _ => return Err(ChifError::ParserError {
                message: "Expected struct name".to_string(),
            }),
        };
        
        self.consume(Token::LeftBrace, "Expected '{' after struct name")?;
        
        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_function(false)?);
        }
        
        self.consume(Token::RightBrace, "Expected '}' after struct methods")?;
        
        Ok(StructImpl {
            struct_name,
            methods,
        })
    }
    
    fn parse_type(&mut self) -> Result<ChifType> {
        match self.advance() {
            Token::Int => Ok(ChifType::Int),
            Token::Float => Ok(ChifType::Float),
            Token::Str => Ok(ChifType::Str),
            Token::Bool => Ok(ChifType::Bool),
            Token::Nil => Ok(ChifType::Nil),
            Token::Pointer => {
                // Check if there's a type specification
                if self.check(&Token::LeftBracket) {
                    self.advance(); // consume '['
                    let inner_type = self.parse_type()?;
                    self.consume(Token::RightBracket, "Expected ']' after pointer type")?;
                    Ok(ChifType::Pointer(Box::new(inner_type)))
                } else {
                    // Generic pointer without specific type
                    Ok(ChifType::Pointer(Box::new(ChifType::Nil)))
                }
            }
            Token::Array => {
                // Support both syntaxes: array[type] and array type[size]
                if self.check(&Token::LeftBracket) {
                    // New syntax: array[type]
                    self.advance(); // consume '['
                    let inner_type = self.parse_type()?;
                    self.consume(Token::RightBracket, "Expected ']' after array type")?;
                    
                    // For function parameters, we don't need size specification
                    Ok(ChifType::Array(Box::new(inner_type), vec![0]))
                } else {
                    // Old syntax: array type[size]
                    let inner_type = self.parse_type()?;
                    let mut dimensions = Vec::new();
                    
                    while self.check(&Token::LeftBracket) {
                        self.advance(); // consume '['
                        if let Token::IntLiteral(size) = self.advance() {
                            dimensions.push(size as usize);
                        } else {
                            return Err(ChifError::ParserError {
                                message: "Expected array size".to_string(),
                            });
                        }
                        self.consume(Token::RightBracket, "Expected ']' after array size")?;
                    }
                    
                    Ok(ChifType::Array(Box::new(inner_type), dimensions))
                }
            }
            Token::List => {
                // Support both syntaxes: list[type] and list type[]
                if self.check(&Token::LeftBracket) {
                    // New syntax: list[type]
                    self.advance(); // consume '['
                    let inner_type = self.parse_type()?;
                    self.consume(Token::RightBracket, "Expected ']' after list type")?;
                    
                    // Check for additional dimensions [][]
                    let mut dimensions = vec![0]; // One dimension by default
                    while self.check(&Token::LeftBracket) {
                        self.advance(); // consume '['
                        self.consume(Token::RightBracket, "Expected ']' for list dimension")?;
                        dimensions.push(0);
                    }
                    
                    Ok(ChifType::List(Box::new(inner_type), dimensions))
                } else {
                    // Old syntax: list type[]
                    let inner_type = self.parse_type()?;
                    let mut dimensions = Vec::new();
                    
                    while self.check(&Token::LeftBracket) {
                        self.advance(); // consume '['
                        self.consume(Token::RightBracket, "Expected ']' for list dimension")?;
                        dimensions.push(0); // Lists don't have fixed sizes
                    }
                    
                    Ok(ChifType::List(Box::new(inner_type), dimensions))
                }
            }
            Token::Map => {
                self.consume(Token::LeftBracket, "Expected '[' after 'map'")?;
                let key_type = self.parse_type()?;
                self.consume(Token::Colon, "Expected ':' in map type")?;
                let value_type = self.parse_type()?;
                self.consume(Token::RightBracket, "Expected ']' after map type")?;
                Ok(ChifType::Map(Box::new(key_type), Box::new(value_type)))
            }
            Token::Identifier(name) => Ok(ChifType::Struct(name)),
            token => Err(ChifError::ParserError {
                message: format!("Expected type, found {:?}", token),
            }),
        }
    }
    
    fn parse_block(&mut self) -> Result<Block> {
        self.consume(Token::LeftBrace, "Expected '{'")?;
        
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.consume(Token::RightBrace, "Expected '}'")?;
        
        Ok(Block { statements })
    }
    
    fn parse_statement(&mut self) -> Result<Statement> {
        match &self.peek() {
            Token::Let | Token::Var => self.parse_var_decl(),
            Token::Array | Token::List => self.parse_var_decl(),
            Token::If => self.parse_if_statement(),
            Token::For => self.parse_for_statement(),
            Token::While => self.parse_while_statement(),
            Token::Switch => self.parse_switch_statement(),
            Token::Ret => self.parse_return_statement(),
            _ => {
                let expr = self.parse_expression()?;
                
                // Check if this is an assignment
                if self.match_token(&Token::Assign) {
                    let value = self.parse_expression()?;
                    self.consume(Token::Semicolon, "Expected ';' after assignment")?;
                    Ok(Statement::Assignment(Assignment {
                        target: expr,
                        value,
                    }))
                } else {
                    self.consume(Token::Semicolon, "Expected ';' after expression")?;
                    Ok(Statement::Expression(expr))
                }
            }
        }
    }
    
    fn parse_var_decl(&mut self) -> Result<Statement> {
        let (is_mutable, collection_type) = match self.advance() {
            Token::Let => (false, None),
            Token::Var => (true, None),
            Token::Array => (false, Some("array")),
            Token::List => (false, Some("list")),
            _ => return Err(ChifError::ParserError {
                message: "Expected variable declaration".to_string(),
            }),
        };
        
        let name = match self.advance() {
            Token::Identifier(name) => name,
            _ => return Err(ChifError::ParserError {
                message: "Expected variable name".to_string(),
            }),
        };
        
        self.consume(Token::Colon, "Expected ':' after variable name")?;
        
        // Parse type - handle collection types specially
        let var_type = if let Some(coll_type) = collection_type {
            match coll_type {
                "array" => {
                    // Parse array name: type[size][size]...
                    let inner_type = self.parse_type()?;
                    let mut dimensions = Vec::new();
                    
                    while self.check(&Token::LeftBracket) {
                        self.advance(); // consume '['
                        if let Token::IntLiteral(size) = self.advance() {
                            dimensions.push(size as usize);
                        } else {
                            return Err(ChifError::ParserError {
                                message: "Expected array size".to_string(),
                            });
                        }
                        self.consume(Token::RightBracket, "Expected ']' after array size")?;
                    }
                    
                    crate::types::ChifType::Array(Box::new(inner_type), dimensions)
                }
                "list" => {
                    // Parse list name: type[]...
                    let inner_type = self.parse_type()?;
                    let mut dimensions = Vec::new();
                    
                    while self.check(&Token::LeftBracket) {
                        self.advance(); // consume '['
                        self.consume(Token::RightBracket, "Expected ']' for list dimension")?;
                        dimensions.push(0); // Lists don't have fixed sizes
                    }
                    
                    crate::types::ChifType::List(Box::new(inner_type), dimensions)
                }
                _ => unreachable!(),
            }
        } else {
            self.parse_type()?
        };
        
        let value = if self.match_token(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume(Token::Semicolon, "Expected ';' after variable declaration")?;
        
        Ok(Statement::VarDecl(VarDecl {
            name,
            var_type,
            value,
            is_mutable,
        }))
    }
    
    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.consume(Token::If, "Expected 'if'")?;
        self.consume(Token::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression()?;
        self.consume(Token::RightParen, "Expected ')' after if condition")?;
        
        let then_block = self.parse_block()?;
        
        let else_block = if self.match_token(&Token::Else) {
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Statement::If(IfStatement {
            condition,
            then_block,
            else_block,
        }))
    }
    
    fn parse_for_statement(&mut self) -> Result<Statement> {
        self.consume(Token::For, "Expected 'for'")?;
        self.consume(Token::LeftParen, "Expected '(' after 'for'")?;
        
        // Parse initialization (variable assignment)
        let init = if !self.check(&Token::Semicolon) {
            // Parse as assignment: i = 0
            let var_name = match self.advance() {
                Token::Identifier(name) => name,
                _ => return Err(ChifError::ParserError {
                    message: "Expected variable name in for loop initialization".to_string(),
                }),
            };
            
            self.consume(Token::Assign, "Expected '=' in for loop initialization")?;
            let value = self.parse_expression()?;
            
            // Create a variable declaration statement
            Some(Box::new(Statement::VarDecl(VarDecl {
                name: var_name,
                var_type: crate::types::ChifType::Int, // Assume int for now
                value: Some(value),
                is_mutable: true,
            })))
        } else {
            None
        };
        
        self.consume(Token::Semicolon, "Expected ';' after for initialization")?;
        
        let condition = if !self.check(&Token::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume(Token::Semicolon, "Expected ';' after for condition")?;
        
        let update = if !self.check(&Token::RightParen) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume(Token::RightParen, "Expected ')' after for clauses")?;
        
        let body = self.parse_block()?;
        
        Ok(Statement::For(ForStatement {
            init,
            condition,
            update,
            body,
        }))
    }
    
    fn parse_while_statement(&mut self) -> Result<Statement> {
        self.consume(Token::While, "Expected 'while'")?;
        self.consume(Token::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.consume(Token::RightParen, "Expected ')' after while condition")?;
        
        let body = self.parse_block()?;
        
        Ok(Statement::While(WhileStatement { condition, body }))
    }
    
    fn parse_switch_statement(&mut self) -> Result<Statement> {
        self.consume(Token::Switch, "Expected 'switch'")?;
        let expr = self.parse_expression()?;
        self.consume(Token::Colon, "Expected ':' after switch expression")?;
        
        let mut cases = Vec::new();
        let mut default_case = None;
        
        while !self.is_at_end() && (self.check(&Token::Case) || self.check(&Token::Default)) {
            if self.match_token(&Token::Case) {
                let value = self.parse_expression()?;
                let body = self.parse_block()?;
                cases.push(SwitchCase { value, body });
            } else if self.match_token(&Token::Default) {
                default_case = Some(self.parse_block()?);
                break;
            }
        }
        
        Ok(Statement::Switch(SwitchStatement {
            expr,
            cases,
            default_case,
        }))
    }
    
    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.consume(Token::Ret, "Expected 'ret'")?;
        
        let value = if !self.check(&Token::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume(Token::Semicolon, "Expected ';' after return statement")?;
        
        Ok(Statement::Return(value))
    }
    
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_or()
    }
    
    fn parse_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and()?;
        
        while self.match_token(&Token::Or) {
            let right = self.parse_and()?;
            expr = Expression::Binary(BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&Token::And) {
            let right = self.parse_equality()?;
            expr = Expression::Binary(BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;
        
        while let Some(op) = self.match_equality_op() {
            let right = self.parse_comparison()?;
            expr = Expression::Binary(BinaryOp {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;
        
        while let Some(op) = self.match_comparison_op() {
            let right = self.parse_term()?;
            expr = Expression::Binary(BinaryOp {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;
        
        while let Some(op) = self.match_term_op() {
            let right = self.parse_factor()?;
            expr = Expression::Binary(BinaryOp {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;
        
        while let Some(op) = self.match_factor_op() {
            let right = self.parse_unary()?;
            expr = Expression::Binary(BinaryOp {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            });
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> Result<Expression> {
        if let Some(op) = self.match_unary_op() {
            let operand = self.parse_unary()?;
            Ok(Expression::Unary(UnaryOp {
                operator: op,
                operand: Box::new(operand),
            }))
        } else if self.match_token(&Token::Reference) {
            let operand = self.parse_unary()?;
            Ok(Expression::Reference(Box::new(operand)))
        } else if self.match_token(&Token::Multiply) {
            // In unary context, * is dereference
            let operand = self.parse_unary()?;
            Ok(Expression::Dereference(Box::new(operand)))
        } else {
            self.parse_postfix()
        }
    }
    
    fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&Token::LeftParen) {
                // Function call
                let mut args = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.consume(Token::RightParen, "Expected ')' after function arguments")?;
                
                if let Expression::Identifier(name) = expr {
                    expr = Expression::Call(FunctionCall { name, args });
                } else {
                    return Err(ChifError::ParserError {
                        message: "Invalid function call".to_string(),
                    });
                }
            } else if self.match_token(&Token::LeftBracket) {
                // Index access
                let mut indices = Vec::new();
                indices.push(self.parse_expression()?);
                self.consume(Token::RightBracket, "Expected ']' after index")?;
                
                while self.match_token(&Token::LeftBracket) {
                    indices.push(self.parse_expression()?);
                    self.consume(Token::RightBracket, "Expected ']' after index")?;
                }
                
                expr = Expression::Index(IndexAccess {
                    object: Box::new(expr),
                    indices,
                });
            } else if self.match_token(&Token::Dot) {
                // Field access or method call
                let field_name = match self.advance() {
                    Token::Identifier(name) => name,
                    _ => return Err(ChifError::ParserError {
                        message: "Expected field or method name after '.'".to_string(),
                    }),
                };
                
                if self.match_token(&Token::LeftParen) {
                    // Method call
                    let mut args = Vec::new();
                    if !self.check(&Token::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(Token::RightParen, "Expected ')' after method arguments")?;
                    
                    expr = Expression::MethodCall(MethodCall {
                        object: Box::new(expr),
                        method: field_name,
                        args,
                    });
                } else {
                    // Field access
                    expr = Expression::FieldAccess(FieldAccess {
                        object: Box::new(expr),
                        field: field_name,
                    });
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> Result<Expression> {
        match self.advance() {
            Token::IntLiteral(value) => Ok(Expression::Literal(ChifValue::Int(value))),
            Token::FloatLiteral(value) => Ok(Expression::Literal(ChifValue::Float(value))),
            Token::StringLiteral(value) => Ok(Expression::Literal(ChifValue::Str(value))),
            Token::BoolLiteral(value) => Ok(Expression::Literal(ChifValue::Bool(value))),
            Token::Nil => Ok(Expression::Literal(ChifValue::Nil)),
            Token::Identifier(name) => {
                // Check if this is a struct literal: StructName { ... }
                if self.check(&Token::LeftBrace) {
                    self.advance(); // consume '{'
                    
                    let mut fields = Vec::new();
                    if !self.check(&Token::RightBrace) {
                        loop {
                            let field_name = match self.advance() {
                                Token::Identifier(field) => field,
                                _ => return Err(ChifError::ParserError {
                                    message: "Expected field name in struct literal".to_string(),
                                }),
                            };
                            
                            self.consume(Token::Assign, "Expected '=' after field name")?;
                            let field_value = self.parse_expression()?;
                            fields.push((field_name, field_value));
                            
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                            // Handle trailing comma
                            if self.check(&Token::RightBrace) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(Token::RightBrace, "Expected '}' after struct fields")?;
                    
                    Ok(Expression::StructLiteral(StructLiteral {
                        struct_name: name,
                        fields,
                    }))
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Token::LeftParen => {
                let expr = self.parse_expression()?;
                self.consume(Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::LeftBracket => {
                // Array literal
                let mut elements = Vec::new();
                if !self.check(&Token::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.consume(Token::RightBracket, "Expected ']' after array elements")?;
                Ok(Expression::ArrayLiteral(elements))
            }
            Token::LeftBrace => {
                // Map literal or struct literal
                if self.check(&Token::StringLiteral("".to_string())) || self.check(&Token::Identifier("".to_string())) {
                    // This is a heuristic - we'll need to improve this
                    let mut pairs = Vec::new();
                    if !self.check(&Token::RightBrace) {
                        loop {
                            let key = self.parse_expression()?;
                            self.consume(Token::Colon, "Expected ':' in map literal")?;
                            let value = self.parse_expression()?;
                            pairs.push((key, value));
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(Token::RightBrace, "Expected '}' after map elements")?;
                    Ok(Expression::MapLiteral(pairs))
                } else {
                    return Err(ChifError::ParserError {
                        message: "Unexpected '{'".to_string(),
                    });
                }
            }
            token => Err(ChifError::ParserError {
                message: format!("Unexpected token: {:?}", token),
            }),
        }
    }
    
    // Helper methods
    fn match_equality_op(&mut self) -> Option<BinaryOperator> {
        match self.peek() {
            Token::Equal => {
                self.advance();
                Some(BinaryOperator::Equal)
            }
            Token::NotEqual => {
                self.advance();
                Some(BinaryOperator::NotEqual)
            }
            _ => None,
        }
    }
    
    fn match_comparison_op(&mut self) -> Option<BinaryOperator> {
        match self.peek() {
            Token::Less => {
                self.advance();
                Some(BinaryOperator::Less)
            }
            Token::Greater => {
                self.advance();
                Some(BinaryOperator::Greater)
            }
            Token::LessEqual => {
                self.advance();
                Some(BinaryOperator::LessEqual)
            }
            Token::GreaterEqual => {
                self.advance();
                Some(BinaryOperator::GreaterEqual)
            }
            _ => None,
        }
    }
    
    fn match_term_op(&mut self) -> Option<BinaryOperator> {
        match self.peek() {
            Token::Plus => {
                self.advance();
                Some(BinaryOperator::Add)
            }
            Token::Minus => {
                self.advance();
                Some(BinaryOperator::Subtract)
            }
            _ => None,
        }
    }
    
    fn match_factor_op(&mut self) -> Option<BinaryOperator> {
        match self.peek() {
            Token::Multiply => {
                self.advance();
                Some(BinaryOperator::Multiply)
            }
            Token::Divide => {
                self.advance();
                Some(BinaryOperator::Divide)
            }
            Token::Modulo => {
                self.advance();
                Some(BinaryOperator::Modulo)
            }
            _ => None,
        }
    }
    
    fn match_unary_op(&mut self) -> Option<UnaryOperator> {
        match self.peek() {
            Token::Not => {
                self.advance();
                Some(UnaryOperator::Not)
            }
            Token::Minus => {
                self.advance();
                Some(UnaryOperator::Minus)
            }
            _ => None,
        }
    }
    
    fn match_token(&mut self, token: &Token) -> bool {
        if std::mem::discriminant(&self.peek()) == std::mem::discriminant(token) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(&self.peek()) == std::mem::discriminant(token)
    }
    
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }
    
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }
    
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
    
    fn consume(&mut self, token: Token, message: &str) -> Result<Token> {
        if std::mem::discriminant(&self.peek()) == std::mem::discriminant(&token) {
            Ok(self.advance())
        } else {
            Err(ChifError::ParserError {
                message: format!("{}, found {:?}", message, self.peek()),
            })
        }
    }
}