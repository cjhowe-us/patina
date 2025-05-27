use super::*;
use serde::{Deserialize, Serialize};

/// Integer represents an integer literal in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Integer {
    pub pos: Position,
    pub value: u64,
}

impl SyntaxNode for Integer {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![]
    }
}

/// Boolean represents a boolean literal in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Boolean {
    pub pos: Position,
    pub value: bool,
}

impl SyntaxNode for Boolean {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![]
    }
}

/// Float represents a floating point literal in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Float {
    pub pos: Position,
    pub value: f64,
}

impl SyntaxNode for Float {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![]
    }
}

/// Null represents a null literal in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Null {
    pub pos: Position,
}

impl SyntaxNode for Null {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![]
    }
}

/// StringLiteral represents a string literal in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringLiteral {
    pub pos: Position,
    pub fragments: Vec<StringFragment>,
}

impl SyntaxNode for StringLiteral {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.fragments
            .iter()
            .map(|f| f as &dyn SyntaxNode)
            .collect()
    }
}

/// StringFragment represents a fragment of a string literal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringFragment {
    pub pos: Position,
    pub text: Option<String>,
    pub interpolation: Option<ExpressionKind>,
}

impl SyntaxNode for StringFragment {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        if let Some(interpolation) = &self.interpolation {
            vec![interpolation as &dyn SyntaxNode]
        } else {
            vec![]
        }
    }
}

impl SyntaxNode for ValueKind {
    fn position(&self) -> &Position {
        match self {
            ValueKind::Integer(i) => i.position(),
            ValueKind::Boolean(b) => b.position(),
            ValueKind::Float(f) => f.position(),
            ValueKind::Null(n) => n.position(),
            ValueKind::String(s) => s.position(),
        }
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        match self {
            ValueKind::Integer(i) => i.children(),
            ValueKind::Boolean(b) => b.children(),
            ValueKind::Float(f) => f.children(),
            ValueKind::Null(n) => n.children(),
            ValueKind::String(s) => s.children(),
        }
    }
}
