//! Components that can be attached to AST entities

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "components_tests.rs"]
mod tests;

/// Source position component for tracking location in source code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub filename: Option<String>,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        }
    }
}

/// Stage information component for multi-stage compilation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stage {
    pub stage_number: u32,
}

/// Type information component added during type checking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeInfo {
    pub type_name: String,
    pub is_resolved: bool,
}

/// Symbol information component for name resolution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub scope_id: usize,
    pub is_defined: bool,
}

/// Comment component for preserving comments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub text: String,
    pub is_doc_comment: bool,
}

/// Annotation component for metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    pub name: String,
    pub arguments: Vec<String>,
}

/// Error component for tracking parse/analysis errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
    pub error_type: ErrorType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorType {
    ParseError,
    TypeError,
    NameError,
    StageError,
}

/// Component wrapper for type-erased storage
pub struct ComponentBox {
    pub component: Box<dyn std::any::Any>,
}

impl ComponentBox {
    pub fn new<T: 'static>(component: T) -> Self {
        Self {
            component: Box::new(component),
        }
    }

    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        self.component.downcast_ref()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.component.downcast_mut()
    }
}
