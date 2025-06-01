//! Patina Language AST and Parser
//!
//! This crate provides the abstract syntax tree representation and parser for the Patina language.
//! The AST uses a component-based architecture for maximum extensibility:
//! - Components store data (source location, hierarchy, analysis results)
//! - Nodes are lightweight entities with IDs
//! - Systems operate on components for traversal and analysis

pub mod analysis;
pub mod ast;
pub mod parser;

#[cfg(test)]
pub mod test_shared;

pub use ast::*;
pub use parser::*;
