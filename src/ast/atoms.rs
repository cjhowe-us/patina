use super::*;
use serde::{Deserialize, Serialize};

/// AtomicType represents a type atom in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtomicType {
    pub pos: Position,
    pub type_expr: Option<Box<TypeKind>>,
}

impl SyntaxNode for AtomicType {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        if let Some(type_expr) = &self.type_expr {
            vec![type_expr.as_ref() as &dyn SyntaxNode]
        } else {
            vec![]
        }
    }
}

/// GroupedExpression represents a parenthesized expression in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupedExpression {
    pub pos: Position,
    pub expression: Option<Box<ExpressionKind>>,
}

impl SyntaxNode for GroupedExpression {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        if let Some(expression) = &self.expression {
            vec![expression.as_ref() as &dyn SyntaxNode]
        } else {
            vec![]
        }
    }
}

/// AtomicValue represents a value used as an atom in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtomicValue {
    pub pos: Position,
    pub value: Option<ValueKind>,
}

impl SyntaxNode for AtomicValue {
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

/// Binding represents an identifier binding in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub pos: Position,
    pub name: String,
}

impl SyntaxNode for Binding {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![]
    }
}

impl SyntaxNode for AtomKind {
    fn position(&self) -> &Position {
        match self {
            AtomKind::Grouped(g) => g.position(),
            AtomKind::Type(t) => t.position(),
            AtomKind::Value(v) => v.position(),
            AtomKind::Binding(b) => b.position(),
        }
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        match self {
            AtomKind::Grouped(g) => g.children(),
            AtomKind::Type(t) => t.children(),
            AtomKind::Value(v) => v.children(),
            AtomKind::Binding(b) => b.children(),
        }
    }
}
