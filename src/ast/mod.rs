use serde::{Deserialize, Serialize};

pub mod atoms;
pub mod expressions;
pub mod statements;
pub mod syntax_nodes;
pub mod types;
pub mod values;
pub mod walk;

pub use atoms::*;
pub use expressions::*;
pub use statements::*;
pub use syntax_nodes::*;
pub use types::*;
pub use values::*;
pub use walk::*;

/// Position information for AST nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub filename: Option<String>,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

/// Base trait for all AST nodes
pub trait SyntaxNode {
    fn position(&self) -> &Position;
    fn children(&self) -> Vec<&dyn SyntaxNode>;
}


/// Enum for all statement types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatementKind {
    Module(Module),
    Function(FunctionStatement),
    Return(ReturnStatement),
    Let(LetStatement),
    Assignment(AssignmentStatement),
    Break(BreakStatement),
    While(WhileStatement),
    For(ForStatement),
    Struct(StructStatement),
    Enum(EnumStatement),
    Interface(InterfaceStatement),
    Impl(ImplStatement),
    Expression(ExpressionStatement),
}

/// Enum for all expression types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionKind {
    If(IfExpression),
    Case(CaseExpression),
    Lambda(Lambda),
    Block(BlockExpression),
    Type(TypeExpression),
    Operation(Operation),
}

/// Enum for all atom types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AtomKind {
    Grouped(GroupedExpression),
    Type(AtomicType),
    Value(AtomicValue),
    Binding(Binding),
}

/// Enum for all value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueKind {
    Integer(Integer),
    Boolean(Boolean),
    Float(Float),
    Null(Null),
    String(StringLiteral),
}

/// Enum for all type expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeKind {
    Pointer(PointerType),
    Array(ArrayType),
    Integer(IntegerType),
    Floating(FloatingType),
    Void(VoidType),
    Self_(SelfType),
    Function(FunctionType),
}
