use super::*;
use serde::{Deserialize, Serialize};

/// PointerType represents a pointer type in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointerType {
    pub pos: Position,
    pub is_owned: bool,
    pub is_raw: bool,
    pub is_mutable: bool,
    pub target: AtomKind,
}

impl SyntaxNode for PointerType {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![&self.target as &dyn SyntaxNode]
    }
}

/// ArrayType represents an array type in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayType {
    pub pos: Position,
    pub dimension: Option<u64>,
    pub element: AtomKind,
}

impl SyntaxNode for ArrayType {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![&self.element as &dyn SyntaxNode]
    }
}

/// IntegerType represents an integer type with specific bit size
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegerType {
    pub pos: Position,
    pub signed: bool,
    pub bit_size: u64,
}

impl SyntaxNode for IntegerType {
    fn position(&self) -> &Position {
        &self.pos
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        vec![]
    }
}

/// FunctionType represents a function type in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionType {
    pub pos: Position,
    pub name: Option<String>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<AtomKind>,
}

impl SyntaxNode for FunctionType {
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

impl SyntaxNode for TypeKind {
    fn position(&self) -> &Position {
        match self {
            TypeKind::Pointer(p) => p.position(),
            TypeKind::Array(a) => a.position(),
            TypeKind::Integer(i) => i.position(),
            TypeKind::Function(f) => f.position(),
        }
    }

    fn children(&self) -> Vec<&dyn SyntaxNode> {
        match self {
            TypeKind::Pointer(p) => p.children(),
            TypeKind::Array(a) => a.children(),
            TypeKind::Integer(i) => i.children(),
            TypeKind::Function(f) => f.children(),
        }
    }
}
