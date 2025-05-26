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
            StatementKind::Break(b) => b.position(),
            StatementKind::Expression(e) => e.position(),
        }
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        match self {
            StatementKind::Module(m) => m.children(),
            StatementKind::Function(f) => f.children(),
            StatementKind::Return(r) => r.children(),
            StatementKind::Let(l) => l.children(),
            StatementKind::Break(b) => b.children(),
            StatementKind::Expression(e) => e.children(),
        }
    }
}
