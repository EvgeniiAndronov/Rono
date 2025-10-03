use crate::types::{ChifType, ChifValue};

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Import(ImportStatement),
    Function(Function),
    Struct(StructDef),
    StructImpl(StructImpl),
}

#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub path: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<ChifType>,
    pub body: Block,
    pub is_main: bool,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: ChifType,
    pub is_reference: bool,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub field_type: ChifType,
}

#[derive(Debug, Clone)]
pub struct StructImpl {
    pub struct_name: String,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl(VarDecl),
    Assignment(Assignment),
    Expression(Expression),
    If(IfStatement),
    For(ForStatement),
    While(WhileStatement),
    Switch(SwitchStatement),
    Return(Option<Expression>),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub var_type: ChifType,
    pub value: Option<Expression>,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: Expression,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct ForStatement {
    pub init: Option<Box<Statement>>,
    pub condition: Option<Expression>,
    pub update: Option<Box<Statement>>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct SwitchStatement {
    pub expr: Expression,
    pub cases: Vec<SwitchCase>,
    pub default_case: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub value: Expression,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(ChifValue),
    Identifier(String),
    Binary(BinaryOp),
    Unary(UnaryOp),
    Call(FunctionCall),
    MethodCall(MethodCall),
    Index(IndexAccess),
    FieldAccess(FieldAccess),
    ArrayLiteral(Vec<Expression>),
    MapLiteral(Vec<(Expression, Expression)>),
    StructLiteral(StructLiteral),
    Reference(Box<Expression>),
    Dereference(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct BinaryOp {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct UnaryOp {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Minus,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub object: Box<Expression>,
    pub method: String,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct IndexAccess {
    pub object: Box<Expression>,
    pub indices: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct FieldAccess {
    pub object: Box<Expression>,
    pub field: String,
}

#[derive(Debug, Clone)]
pub struct StructLiteral {
    pub struct_name: String,
    pub fields: Vec<(String, Expression)>,
}