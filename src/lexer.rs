use crate::error::{ChifError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Chif,
    Let,
    Var,
    Array,
    List,
    Map,
    Fn,
    FnFor,
    Struct,
    If,
    Else,
    For,
    While,
    Switch,
    Case,
    Default,
    Ret,
    Import,
    As,
    
    // Types
    Int,
    Float,
    Str,
    Bool,
    Nil,
    Pointer,
    
    // Identifiers and literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Assign,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Not,
    Reference,
    Dereference,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    
    // Special
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace();
            if self.is_at_end() {
                break;
            }
            
            let token = self.next_token()?;
            tokens.push(token);
        }
        
        tokens.push(Token::Eof);
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token> {
        let ch = self.advance();
        
        match ch {
            '(' => Ok(Token::LeftParen),
            ')' => Ok(Token::RightParen),
            '{' => Ok(Token::LeftBrace),
            '}' => Ok(Token::RightBrace),
            '[' => Ok(Token::LeftBracket),
            ']' => Ok(Token::RightBracket),
            ';' => Ok(Token::Semicolon),
            ':' => Ok(Token::Colon),
            ',' => Ok(Token::Comma),
            '.' => Ok(Token::Dot),
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => {
                // In this simple implementation, we'll treat * as multiply by default
                // The parser will need to determine context for dereference
                Ok(Token::Multiply)
            },
            '/' => Ok(Token::Divide),
            '%' => Ok(Token::Modulo),
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    Ok(Token::And)
                } else {
                    Ok(Token::Reference)
                }
            },
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    Ok(Token::Or)
                } else {
                    Err(ChifError::LexerError {
                        line: self.line,
                        column: self.column,
                        message: "Unexpected character '|'".to_string(),
                    })
                }
            },
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::NotEqual)
                } else {
                    Ok(Token::Not)
                }
            },
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::Equal)
                } else {
                    Ok(Token::Assign)
                }
            },
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::LessEqual)
                } else {
                    Ok(Token::Less)
                }
            },
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::GreaterEqual)
                } else {
                    Ok(Token::Greater)
                }
            },
            '"' => self.string_literal(),
            _ if ch.is_ascii_digit() => self.number_literal(ch),
            _ if ch.is_ascii_alphabetic() || ch == '_' => self.identifier_or_keyword(ch),
            _ => Err(ChifError::LexerError {
                line: self.line,
                column: self.column,
                message: format!("Unexpected character '{}'", ch),
            }),
        }
    }
    
    fn string_literal(&mut self) -> Result<Token> {
        let mut value = String::new();
        
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance(); // consume closing quote
                return Ok(Token::StringLiteral(value));
            }
            
            if ch == '\\' {
                self.advance(); // consume backslash
                match self.peek() {
                    Some('n') => {
                        value.push('\n');
                        self.advance();
                    },
                    Some('t') => {
                        value.push('\t');
                        self.advance();
                    },
                    Some('r') => {
                        value.push('\r');
                        self.advance();
                    },
                    Some('\\') => {
                        value.push('\\');
                        self.advance();
                    },
                    Some('"') => {
                        value.push('"');
                        self.advance();
                    },
                    _ => {
                        return Err(ChifError::LexerError {
                            line: self.line,
                            column: self.column,
                            message: "Invalid escape sequence".to_string(),
                        });
                    }
                }
            } else {
                value.push(self.advance());
            }
        }
        
        Err(ChifError::LexerError {
            line: self.line,
            column: self.column,
            message: "Unterminated string literal".to_string(),
        })
    }
    
    fn number_literal(&mut self, first_digit: char) -> Result<Token> {
        let mut value = String::new();
        value.push(first_digit);
        
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                value.push(self.advance());
            } else {
                break;
            }
        }
        
        // Check for float
        if self.peek() == Some('.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            value.push(self.advance()); // consume '.'
            
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    value.push(self.advance());
                } else {
                    break;
                }
            }
            
            let float_val = value.parse::<f64>().map_err(|_| ChifError::LexerError {
                line: self.line,
                column: self.column,
                message: "Invalid float literal".to_string(),
            })?;
            
            Ok(Token::FloatLiteral(float_val))
        } else {
            let int_val = value.parse::<i64>().map_err(|_| ChifError::LexerError {
                line: self.line,
                column: self.column,
                message: "Invalid integer literal".to_string(),
            })?;
            
            Ok(Token::IntLiteral(int_val))
        }
    }
    
    fn identifier_or_keyword(&mut self, first_char: char) -> Result<Token> {
        let mut value = String::new();
        value.push(first_char);
        
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                value.push(self.advance());
            } else {
                break;
            }
        }
        
        let token = match value.as_str() {
            "chif" => Token::Chif,
            "let" => Token::Let,
            "var" => Token::Var,
            "array" => Token::Array,
            "list" => Token::List,
            "map" => Token::Map,
            "fn" => Token::Fn,
            "fn_for" => Token::FnFor,
            "struct" => Token::Struct,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "switch" => Token::Switch,
            "case" => Token::Case,
            "default" => Token::Default,
            "ret" => Token::Ret,
            "import" => Token::Import,
            "as" => Token::As,
            "int" => Token::Int,
            "float" => Token::Float,
            "str" => Token::Str,
            "bool" => Token::Bool,
            "nil" => Token::Nil,
            "pointer" => Token::Pointer,
            "true" => Token::BoolLiteral(true),
            "false" => Token::BoolLiteral(false),
            _ => Token::Identifier(value),
        };
        
        Ok(token)
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn advance(&mut self) -> char {
        let ch = self.input[self.position];
        self.position += 1;
        self.column += 1;
        ch
    }
    
    fn peek(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }
    
    fn peek_next(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}