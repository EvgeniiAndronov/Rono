#[cfg(test)]
mod tests {
    use crate::semantic::SemanticAnalyzer;
    use crate::ast::*;
    use crate::types::{ChifType, ChifValue};

    #[test]
    fn test_basic_semantic_analysis() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Create a simple program with a function
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "test_func".to_string(),
                    params: vec![
                        Parameter {
                            name: "x".to_string(),
                            param_type: ChifType::Int,
                            is_reference: false,
                        }
                    ],
                    return_type: Some(ChifType::Int),
                    body: Block {
                        statements: vec![
                            Statement::Return(Some(Expression::Identifier("x".to_string())))
                        ]
                    },
                    is_main: false,
                })
            ]
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok(), "Semantic analysis should succeed for valid program");
    }
    
    #[test]
    fn test_undefined_symbol_error() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Create a program that uses an undefined variable
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "test_func".to_string(),
                    params: vec![],
                    return_type: Some(ChifType::Int),
                    body: Block {
                        statements: vec![
                            Statement::Return(Some(Expression::Identifier("undefined_var".to_string())))
                        ]
                    },
                    is_main: false,
                })
            ]
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_err(), "Semantic analysis should fail for undefined symbol");
    }
    
    #[test]
    fn test_type_mismatch_error() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Create a program with type mismatch
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "test_func".to_string(),
                    params: vec![],
                    return_type: Some(ChifType::Int),
                    body: Block {
                        statements: vec![
                            Statement::VarDecl(VarDecl {
                                name: "x".to_string(),
                                var_type: ChifType::Int,
                                value: Some(Expression::Literal(ChifValue::Str("hello".to_string()))),
                                is_mutable: false,
                            })
                        ]
                    },
                    is_main: false,
                })
            ]
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_err(), "Semantic analysis should fail for type mismatch");
    }
    
    #[test]
    fn test_binary_operation_types() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Create a program with binary operations
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "test_func".to_string(),
                    params: vec![],
                    return_type: Some(ChifType::Int),
                    body: Block {
                        statements: vec![
                            Statement::VarDecl(VarDecl {
                                name: "x".to_string(),
                                var_type: ChifType::Int,
                                value: Some(Expression::Binary(BinaryOp {
                                    left: Box::new(Expression::Literal(ChifValue::Int(5))),
                                    operator: BinaryOperator::Add,
                                    right: Box::new(Expression::Literal(ChifValue::Int(3))),
                                })),
                                is_mutable: false,
                            }),
                            Statement::Return(Some(Expression::Identifier("x".to_string())))
                        ]
                    },
                    is_main: false,
                })
            ]
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok(), "Semantic analysis should succeed for valid binary operations");
    }
    
    #[test]
    fn test_function_return_validation() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Create a function that doesn't return a value when it should
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "test_func".to_string(),
                    params: vec![],
                    return_type: Some(ChifType::Int),
                    body: Block {
                        statements: vec![
                            Statement::VarDecl(VarDecl {
                                name: "x".to_string(),
                                var_type: ChifType::Int,
                                value: Some(Expression::Literal(ChifValue::Int(42))),
                                is_mutable: false,
                            })
                            // Missing return statement
                        ]
                    },
                    is_main: false,
                })
            ]
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_err(), "Semantic analysis should fail for function without return");
    }
    
    #[test]
    fn test_conditional_return_validation() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Create a function with conditional returns
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "test_func".to_string(),
                    params: vec![
                        Parameter {
                            name: "condition".to_string(),
                            param_type: ChifType::Bool,
                            is_reference: false,
                        }
                    ],
                    return_type: Some(ChifType::Int),
                    body: Block {
                        statements: vec![
                            Statement::If(IfStatement {
                                condition: Expression::Identifier("condition".to_string()),
                                then_block: Block {
                                    statements: vec![
                                        Statement::Return(Some(Expression::Literal(ChifValue::Int(1))))
                                    ]
                                },
                                else_block: Some(Block {
                                    statements: vec![
                                        Statement::Return(Some(Expression::Literal(ChifValue::Int(0))))
                                    ]
                                }),
                            })
                        ]
                    },
                    is_main: false,
                })
            ]
        };
        
        let result = analyzer.analyze(&program);
        assert!(result.is_ok(), "Semantic analysis should succeed for function with returns in all paths");
    }
}