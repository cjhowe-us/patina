use super::*;
use serde::{Deserialize, Serialize};

/// Module represents a module declaration in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub pos: Position,
    pub name: String,
    pub body: Block,
}

impl SyntaxNode for Module {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![&self.body as &dyn SyntaxNode]
    }
}

/// FunctionStatement represents a function declaration in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionStatement {
    pub pos: Position,
    pub is_static: bool,
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<AtomKind>,
    pub block: Block,
}

impl SyntaxNode for FunctionStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        for param in &self.parameters {
            children.push(param as &dyn SyntaxNode);
        }
        if let Some(return_type) = &self.return_type {
            children.push(return_type as &dyn SyntaxNode);
        }
        children.push(&self.block as &dyn SyntaxNode);
        children
    }
}

/// FunctionParameter represents a parameter in a function declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParameter {
    pub pos: Position,
    pub name: String,
    pub param_type: AtomKind,
}

impl SyntaxNode for FunctionParameter {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![&self.param_type as &dyn SyntaxNode]
    }
}

/// ReturnStatement represents a return statement in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub pos: Position,
    pub label: Option<String>,
    pub value: Option<ExpressionKind>,
}

impl SyntaxNode for ReturnStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        if let Some(value) = &self.value {
            vec![value as &dyn SyntaxNode]
        } else {
            vec![]
        }
    }
}

/// LetStatement represents a variable declaration in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetStatement {
    pub pos: Position,
    pub is_static: bool,
    pub is_mutable: bool,
    pub name: String,
    pub var_type: Option<AtomKind>,
    pub value: Option<ExpressionKind>,
}

impl SyntaxNode for LetStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        if let Some(var_type) = &self.var_type {
            children.push(var_type as &dyn SyntaxNode);
        }
        if let Some(value) = &self.value {
            children.push(value as &dyn SyntaxNode);
        }
        children
    }
}

/// AssignmentStatement represents an assignment to an existing variable in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentStatement {
    pub pos: Position,
    pub target: ExpressionKind,
    pub value: ExpressionKind,
}

impl SyntaxNode for AssignmentStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![
            &self.target as &dyn SyntaxNode,
            &self.value as &dyn SyntaxNode,
        ]
    }
}

/// BreakStatement represents a break statement in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BreakStatement {
    pub pos: Position,
    pub label: Option<String>,
    pub value: Option<ExpressionKind>,
}

impl SyntaxNode for BreakStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        if let Some(value) = &self.value {
            vec![value as &dyn SyntaxNode]
        } else {
            vec![]
        }
    }
}

/// WhileStatement represents a while loop in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileStatement {
    pub pos: Position,
    pub condition: ExpressionKind,
    pub body: Block,
}

impl SyntaxNode for WhileStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![
            &self.condition as &dyn SyntaxNode,
            &self.body as &dyn SyntaxNode,
        ]
    }
}

/// ForStatement represents a for loop in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForStatement {
    pub pos: Position,
    pub initializer: Option<Box<StatementKind>>,
    pub condition: Option<ExpressionKind>,
    pub increment: Option<ExpressionKind>,
    pub variable: Option<String>,
    pub iterable: Option<ExpressionKind>,
    pub body: Block,
}

impl SyntaxNode for ForStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        if let Some(init) = &self.initializer {
            children.push(&**init as &dyn SyntaxNode);
        }
        if let Some(condition) = &self.condition {
            children.push(condition as &dyn SyntaxNode);
        }
        if let Some(increment) = &self.increment {
            children.push(increment as &dyn SyntaxNode);
        }
        if let Some(iterable) = &self.iterable {
            children.push(iterable as &dyn SyntaxNode);
        }
        children.push(&self.body as &dyn SyntaxNode);
        children
    }
}

/// StructStatement represents a struct declaration in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructStatement {
    pub pos: Position,
    pub name: String,
    pub generic_parameters: Vec<String>,
    pub fields: Vec<StructField>,
}

impl SyntaxNode for StructStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.fields.iter().map(|f| f as &dyn SyntaxNode).collect()
    }
}

/// StructField represents a field in a struct declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub pos: Position,
    pub name: String,
    pub field_type: AtomKind,
}

impl SyntaxNode for StructField {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![&self.field_type as &dyn SyntaxNode]
    }
}

/// EnumStatement represents an enum declaration in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumStatement {
    pub pos: Position,
    pub name: String,
    pub generic_parameters: Vec<String>,
    pub variants: Vec<EnumVariant>,
}

impl SyntaxNode for EnumStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.variants.iter().map(|v| v as &dyn SyntaxNode).collect()
    }
}

/// EnumVariant represents a variant in an enum declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub pos: Position,
    pub name: String,
    pub data: Option<Vec<AtomKind>>,
}

impl SyntaxNode for EnumVariant {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        if let Some(data) = &self.data {
            data.iter().map(|d| d as &dyn SyntaxNode).collect()
        } else {
            vec![]
        }
    }
}

/// InterfaceStatement represents an interface declaration in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceStatement {
    pub pos: Position,
    pub name: String,
    pub generic_parameters: Vec<String>,
    pub methods: Vec<InterfaceMethod>,
}

impl SyntaxNode for InterfaceStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.methods.iter().map(|m| m as &dyn SyntaxNode).collect()
    }
}

/// InterfaceMethod represents a method signature in an interface
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceMethod {
    pub pos: Position,
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<AtomKind>,
}

impl SyntaxNode for InterfaceMethod {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        for param in &self.parameters {
            children.push(param as &dyn SyntaxNode);
        }
        if let Some(return_type) = &self.return_type {
            children.push(return_type as &dyn SyntaxNode);
        }
        children
    }
}

/// ImplStatement represents an impl block in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplStatement {
    pub pos: Position,
    pub target_type: String,
    pub interface_name: Option<String>,
    pub generic_parameters: Vec<String>,
    pub methods: Vec<FunctionStatement>,
}

impl SyntaxNode for ImplStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.methods.iter().map(|m| m as &dyn SyntaxNode).collect()
    }
}

/// ExpressionStatement represents an expression used as a statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionStatement {
    pub pos: Position,
    pub expression: ExpressionKind,
}

impl SyntaxNode for ExpressionStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![&self.expression as &dyn SyntaxNode]
    }
}

impl SyntaxNode for StatementKind {
    fn position(&self) -> &Position {
        match self {
            StatementKind::Module(m) => m.position(),
            StatementKind::Function(f) => f.position(),
            StatementKind::Return(r) => r.position(),
            StatementKind::Let(l) => l.position(),
            StatementKind::Assignment(a) => a.position(),
            StatementKind::Break(b) => b.position(),
            StatementKind::While(w) => w.position(),
            StatementKind::For(f) => f.position(),
            StatementKind::Struct(s) => s.position(),
            StatementKind::Enum(e) => e.position(),
            StatementKind::Interface(i) => i.position(),
            StatementKind::Impl(i) => i.position(),
            StatementKind::Expression(e) => e.position(),
        }
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        match self {
            StatementKind::Module(m) => m.children(),
            StatementKind::Function(f) => f.children(),
            StatementKind::Return(r) => r.children(),
            StatementKind::Let(l) => l.children(),
            StatementKind::Assignment(a) => a.children(),
            StatementKind::Break(b) => b.children(),
            StatementKind::While(w) => w.children(),
            StatementKind::For(f) => f.children(),
            StatementKind::Struct(s) => s.children(),
            StatementKind::Enum(e) => e.children(),
            StatementKind::Interface(i) => i.children(),
            StatementKind::Impl(i) => i.children(),
            StatementKind::Expression(e) => e.children(),
        }
    }
}
