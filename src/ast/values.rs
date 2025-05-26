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
            ValueKind::String(s) => s.position(),
        }
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        match self {
            ValueKind::Integer(i) => i.children(),
            ValueKind::String(s) => s.children(),
        }
    }
}
