//! Patina Language AST and Parser
//!
//! This crate provides the abstract syntax tree representation and parser for the Patina language.
//! The AST is organized into different node types that implement specific traits:
//! - `SyntaxNode`: Base trait for all AST nodes
//! - `Statement`: Trait for language statements
//! - `Expression`: Trait for expressions
//! - `Atom`: Trait for atomic expressions
//! - `Value`: Trait for literal values
//! - `Type`: Trait for type values

pub mod ast;
pub mod parser;
pub mod visitor;
pub mod ast_ecs;
// pub mod parser_ecs; // TODO: Fix rule names

#[cfg(test)]
pub mod test_shared;

#[cfg(test)]
mod comparison;

#[cfg(test)]
mod test_ecs;

#[cfg(test)]
mod test_ecs_coverage;

pub use ast::*;
pub use parser::*;
