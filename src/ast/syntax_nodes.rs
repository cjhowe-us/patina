use super::*;
use serde::{Deserialize, Serialize};

/// Root represents the root of an AST tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub pos: Position,
    pub statements: Vec<DecoratedStatement>,
}

impl SyntaxNode for Root {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.statements.iter().map(|s| s as &dyn SyntaxNode).collect()
    }
}

/// Block represents a block of statements in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub pos: Position,
    pub label: Option<String>,
    pub statements: Vec<DecoratedStatement>,
}

impl SyntaxNode for Block {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.statements.iter().map(|s| s as &dyn SyntaxNode).collect()
    }
}

/// DecoratedStatement represents a statement with comments and annotations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecoratedStatement {
    pub pos: Position,
    pub comments: Vec<Comment>,
    pub annotations: Vec<Annotation>,
    pub statement: StatementKind,
}

impl SyntaxNode for DecoratedStatement {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        let mut children: Vec<&dyn SyntaxNode> = Vec::new();
        for comment in &self.comments {
            children.push(comment as &dyn SyntaxNode);
        }
        for annotation in &self.annotations {
            children.push(annotation as &dyn SyntaxNode);
        }
        children.push(&self.statement as &dyn SyntaxNode);
        children
    }
}

/// Comment represents a comment in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub pos: Position,
    pub text: Option<String>,
    pub nested_comments: Vec<Comment>,
}

impl SyntaxNode for Comment {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.nested_comments.iter().map(|c| c as &dyn SyntaxNode).collect()
    }
}

/// Annotation represents an annotation in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub pos: Position,
    pub name: String,
    pub arguments: Vec<ExpressionKind>,
}

impl SyntaxNode for Annotation {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        self.arguments.iter().map(|a| a as &dyn SyntaxNode).collect()
    }
}
