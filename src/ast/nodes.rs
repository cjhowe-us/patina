//! AST node structures with child nodes stored directly
//! Each node has an entity field for its ECS entity ID

use hecs::Entity;
use crate::ast::AstNode;

#[cfg(test)]
#[path = "nodes_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "nodes_additional_tests.rs"]
mod additional_tests;

/// Root node of the AST containing top-level statements
#[derive(Debug, Clone)]
pub struct Root {
    pub entity: Entity,
    pub statements: Vec<Statement>,
}

impl AstNode for Root {
    fn entity_id(&self) -> Entity {
        self.entity
    }
}

/// Statement types in the language
#[derive(Debug, Clone)]
pub enum Statement {
    Module(Module),
    Function(Function),
    Let(Let),
    Return(Return),
    Expression(Expression),
    While(While),
    For(For),
    Break(Break),
    Block(Block),
    Static(Static),
}

impl AstNode for Statement {
    fn entity_id(&self) -> Entity {
        match self {
            Statement::Module(m) => m.entity,
            Statement::Function(f) => f.entity,
            Statement::Let(l) => l.entity,
            Statement::Return(r) => r.entity,
            Statement::Expression(e) => e.entity_id(),
            Statement::While(w) => w.entity,
            Statement::For(f) => f.entity,
            Statement::Break(b) => b.entity,
            Statement::Block(b) => b.entity,
            Statement::Static(s) => s.entity,
        }
    }
}

/// Module declaration
#[derive(Debug, Clone)]
pub struct Module {
    pub entity: Entity,
    pub name: String,
    pub statements: Vec<Statement>,
}

/// Function declaration
#[derive(Debug, Clone)]
pub struct Function {
    pub entity: Entity,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Expression>,
    pub body: Option<Block>,
    pub is_static: bool,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub entity: Entity,
    pub name: String,
    pub type_expr: Expression,
}

impl AstNode for Parameter {
    fn entity_id(&self) -> Entity {
        self.entity
    }
}

/// Let statement for variable declarations
#[derive(Debug, Clone)]
pub struct Let {
    pub entity: Entity,
    pub pattern: Pattern,
    pub type_expr: Option<Expression>,
    pub value: Option<Expression>,
    pub is_mutable: bool,
    pub is_static: bool,
}

/// Pattern for destructuring
#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(Identifier),
    Tuple(TuplePattern),
    Struct(StructPattern),
}

impl AstNode for Pattern {
    fn entity_id(&self) -> Entity {
        match self {
            Pattern::Identifier(i) => i.entity,
            Pattern::Tuple(t) => t.entity,
            Pattern::Struct(s) => s.entity,
        }
    }
}

/// Simple identifier pattern
#[derive(Debug, Clone)]
pub struct Identifier {
    pub entity: Entity,
    pub name: String,
}

/// Tuple pattern for destructuring
#[derive(Debug, Clone)]
pub struct TuplePattern {
    pub entity: Entity,
    pub patterns: Vec<Pattern>,
}

/// Struct pattern for destructuring
#[derive(Debug, Clone)]
pub struct StructPattern {
    pub entity: Entity,
    pub name: String,
    pub fields: Vec<(String, Pattern)>,
}

/// Return statement
#[derive(Debug, Clone)]
pub struct Return {
    pub entity: Entity,
    pub label: Option<String>,
    pub value: Option<Expression>,
}

/// While loop
#[derive(Debug, Clone)]
pub struct While {
    pub entity: Entity,
    pub label: Option<String>,
    pub condition: Expression,
    pub body: Block,
}

/// For loop
#[derive(Debug, Clone)]
pub struct For {
    pub entity: Entity,
    pub label: Option<String>,
    pub init: Option<Box<Statement>>,
    pub condition: Option<Expression>,
    pub update: Option<Expression>,
    pub body: Block,
}

/// Break statement
#[derive(Debug, Clone)]
pub struct Break {
    pub entity: Entity,
    pub label: Option<String>,
    pub value: Option<Expression>,
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub entity: Entity,
    pub label: Option<String>,
    pub statements: Vec<Statement>,
}

/// Static statement - boundary between compile-time and runtime
#[derive(Debug, Clone)]
pub struct Static {
    pub entity: Entity,
    pub statements: Vec<Statement>,
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expression {
    If(Box<If>),
    Case(Box<Case>),
    Lambda(Box<Lambda>),
    Operation(Operation),
    Call(Call),
    Index(Index),
    Field(Field),
    Literal(Literal),
    Identifier(Identifier),
    Block(Block),
    Grouped(Box<Expression>),
}

impl AstNode for Expression {
    fn entity_id(&self) -> Entity {
        match self {
            Expression::If(i) => i.entity,
            Expression::Case(c) => c.entity,
            Expression::Lambda(l) => l.entity,
            Expression::Operation(o) => o.entity,
            Expression::Call(c) => c.entity,
            Expression::Index(i) => i.entity,
            Expression::Field(f) => f.entity,
            Expression::Literal(l) => l.entity_id(),
            Expression::Identifier(i) => i.entity,
            Expression::Block(b) => b.entity,
            Expression::Grouped(e) => e.entity_id(),
        }
    }
}

/// If expression
#[derive(Debug, Clone)]
pub struct If {
    pub entity: Entity,
    pub branches: Vec<IfBranch>,
    pub else_branch: Option<Box<Expression>>,
}

/// Branch of an if expression
#[derive(Debug, Clone)]
pub struct IfBranch {
    pub entity: Entity,
    pub condition: Expression,
    pub body: Expression,
}

impl AstNode for IfBranch {
    fn entity_id(&self) -> Entity {
        self.entity
    }
}

/// Case/match expression
#[derive(Debug, Clone)]
pub struct Case {
    pub entity: Entity,
    pub value: Box<Expression>,
    pub branches: Vec<CaseBranch>,
}

/// Branch of a case expression
#[derive(Debug, Clone)]
pub struct CaseBranch {
    pub entity: Entity,
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
}

impl AstNode for CaseBranch {
    fn entity_id(&self) -> Entity {
        self.entity
    }
}

/// Lambda expression
#[derive(Debug, Clone)]
pub struct Lambda {
    pub entity: Entity,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Box<Expression>>,
    pub body: Box<Expression>,
}

/// Operation expression storing operations in parse order
#[derive(Debug, Clone)]
pub struct Operation {
    pub entity: Entity,
    pub elements: Vec<OperationElement>,
}

/// Element in an operation sequence
#[derive(Debug, Clone)]
pub enum OperationElement {
    Operand(Expression),
    Operator(String),
}

/// Function call
#[derive(Debug, Clone)]
pub struct Call {
    pub entity: Entity,
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}

/// Array/slice indexing
#[derive(Debug, Clone)]
pub struct Index {
    pub entity: Entity,
    pub object: Box<Expression>,
    pub index: Box<Expression>,
}

/// Field access
#[derive(Debug, Clone)]
pub struct Field {
    pub entity: Entity,
    pub object: Box<Expression>,
    pub field: String,
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    Integer(IntegerLiteral),
    Float(FloatLiteral),
    String(StringLiteral),
    Boolean(BooleanLiteral),
    Null(NullLiteral),
}

impl AstNode for Literal {
    fn entity_id(&self) -> Entity {
        match self {
            Literal::Integer(i) => i.entity,
            Literal::Float(f) => f.entity,
            Literal::String(s) => s.entity,
            Literal::Boolean(b) => b.entity,
            Literal::Null(n) => n.entity,
        }
    }
}

/// Integer literal
#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    pub entity: Entity,
    pub value: i64,
}

/// Float literal
#[derive(Debug, Clone)]
pub struct FloatLiteral {
    pub entity: Entity,
    pub value: f64,
}

/// String literal with interpolation support
#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub entity: Entity,
    pub fragments: Vec<StringFragment>,
}

/// Fragment of a string literal
#[derive(Debug, Clone)]
pub enum StringFragment {
    Text(String),
    Interpolation(Expression),
}

/// Boolean literal
#[derive(Debug, Clone)]
pub struct BooleanLiteral {
    pub entity: Entity,
    pub value: bool,
}

/// Null literal
#[derive(Debug, Clone)]
pub struct NullLiteral {
    pub entity: Entity,
}