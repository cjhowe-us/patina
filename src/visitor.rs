use std::error::Error;
use std::fmt;

use crate::ast::SyntaxNode;

/// Visitor trait for traversing AST nodes.
///
/// This trait provides methods for entering and exiting nodes during traversal,
/// allowing for flexible AST processing patterns.
pub trait Visitor<Input, Output> {
    /// Called when entering a node during traversal.
    ///
    /// # Arguments
    /// * `node` - The AST node being entered
    /// * `request` - Optional input data for this node
    ///
    /// # Returns
    /// A vector of input data for each child node, or an error
    fn enter(&mut self, node: &dyn SyntaxNode, request: Option<&Input>) -> Result<Vec<Option<Input>>, Box<dyn Error>>;

    /// Called when exiting a node during traversal.
    ///
    /// # Arguments
    /// * `node` - The AST node being exited
    /// * `children_outputs` - Results from processing child nodes
    ///
    /// # Returns
    /// The output for this node, or an error
    fn exit(&mut self, node: &dyn SyntaxNode, children_outputs: Vec<Option<Output>>) -> Result<Option<Output>, Box<dyn Error>>;
}

/// A collection of errors that occurred during AST traversal.
#[derive(Debug)]
pub struct ErrorList {
    pub errors: Vec<Box<dyn Error>>,
}

impl ErrorList {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn push(&mut self, error: Box<dyn Error>) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl fmt::Display for ErrorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.errors.is_empty() {
            return write!(f, "No errors");
        }

        writeln!(f, "One or more errors occurred:")?;
        for error in &self.errors {
            writeln!(f, "  - {}", error)?;
        }
        Ok(())
    }
}

impl Error for ErrorList {}

/// Walk traverses an AST node recursively, calling the visitor's enter and exit methods
/// for each node encountered.
///
/// # Arguments
/// * `visitor` - The visitor to use for traversal
/// * `input` - Optional input data for the root node
/// * `node` - The root AST node to traverse
///
/// # Returns
/// The output from processing the root node, or an error
pub fn walk<Input, Output>(
    visitor: &mut dyn Visitor<Input, Output>,
    input: Option<&Input>,
    node: &dyn SyntaxNode,
) -> Result<Option<Output>, Box<dyn Error>> {
    let child_inputs = visitor.enter(node, input)?;

    let children = node.children();
    let mut children_results = Vec::with_capacity(children.len());
    let mut errors = ErrorList::new();

    for (i, child) in children.iter().enumerate() {
        let child_input = if i < child_inputs.len() {
            child_inputs[i].as_ref()
        } else {
            None
        };

        match walk(visitor, child_input, *child) {
            Ok(result) => children_results.push(result),
            Err(error) => {
                errors.push(error);
                children_results.push(None);
            }
        }
    }

    if !errors.is_empty() {
        return Err(Box::new(errors));
    }

    visitor.exit(node, children_results)
}

/// A simple visitor that collects all nodes of a specific type.
pub struct NodeCollector<T> {
    pub nodes: Vec<T>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NodeCollector<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Default for NodeCollector<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A visitor that prints the AST structure for debugging.
pub struct DebugPrinter {
    pub depth: usize,
    pub output: String,
}

impl DebugPrinter {
    pub fn new() -> Self {
        Self {
            depth: 0,
            output: String::new(),
        }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.depth)
    }
}

impl Default for DebugPrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor<(), ()> for DebugPrinter {
    fn enter(&mut self, node: &dyn SyntaxNode, _request: Option<&()>) -> Result<Vec<Option<()>>, Box<dyn Error>> {
        let position = node.position();
        self.output.push_str(&format!(
            "{}Node at {}:{} (offset: {})\n",
            self.indent(),
            position.line,
            position.column,
            position.offset
        ));

        self.depth += 1;
        let children_count = node.children().len();
        Ok(vec![Some(()); children_count])
    }

    fn exit(&mut self, _node: &dyn SyntaxNode, _children_outputs: Vec<Option<()>>) -> Result<Option<()>, Box<dyn Error>> {
        self.depth -= 1;
        Ok(Some(()))
    }
}

/// A visitor that validates AST node relationships and constraints.
pub struct ValidationVisitor {
    pub errors: Vec<String>,
}

impl ValidationVisitor {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for ValidationVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor<(), ()> for ValidationVisitor {
    fn enter(&mut self, node: &dyn SyntaxNode, _request: Option<&()>) -> Result<Vec<Option<()>>, Box<dyn Error>> {
        // Basic validation - ensure position is valid
        let position = node.position();
        if position.line == 0 {
            self.errors.push("Invalid line number (should be 1-based)".to_string());
        }
        if position.column == 0 {
            self.errors.push("Invalid column number (should be 1-based)".to_string());
        }

        let children_count = node.children().len();
        Ok(vec![Some(()); children_count])
    }

    fn exit(&mut self, _node: &dyn SyntaxNode, _children_outputs: Vec<Option<()>>) -> Result<Option<()>, Box<dyn Error>> {
        Ok(Some(()))
    }
}

/// A visitor that counts nodes by type.
pub struct NodeCounter {
    pub statement_count: usize,
    pub expression_count: usize,
    pub atom_count: usize,
    pub value_count: usize,
    pub type_count: usize,
}

impl NodeCounter {
    pub fn new() -> Self {
        Self {
            statement_count: 0,
            expression_count: 0,
            atom_count: 0,
            value_count: 0,
            type_count: 0,
        }
    }

    pub fn total_count(&self) -> usize {
        self.statement_count + self.expression_count + self.atom_count + self.value_count + self.type_count
    }
}

impl Default for NodeCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor<(), ()> for NodeCounter {
    fn enter(&mut self, _node: &dyn SyntaxNode, _request: Option<&()>) -> Result<Vec<Option<()>>, Box<dyn Error>> {
        // Note: In a real implementation, we would need to downcast the node
        // to determine its specific type. This is a simplified version.

        let children_count = _node.children().len();
        Ok(vec![Some(()); children_count])
    }

    fn exit(&mut self, _node: &dyn SyntaxNode, _children_outputs: Vec<Option<()>>) -> Result<Option<()>, Box<dyn Error>> {
        // In a real implementation, we would increment counters based on node type
        Ok(Some(()))
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::ast::{Root, Position};

    #[test]
    fn test_debug_printer() {
        let root = Root {
            statements: Vec::new(),
            pos: Position { line: 1, column: 1, offset: 0 },
        };

        let mut printer = DebugPrinter::new();
        let result = walk(&mut printer, None, &root);

        assert!(result.is_ok());
        assert!(printer.output.contains("Node at 1:1"));
    }

    #[test]
    fn test_validation_visitor() {
        let root = Root {
            statements: Vec::new(),
            pos: Position { line: 1, column: 1, offset: 0 },
        };

        let mut validator = ValidationVisitor::new();
        let result = walk(&mut validator, None, &root);

        assert!(result.is_ok());
        assert!(!validator.has_errors());
    }

    #[test]
    fn test_error_list() {
        let mut errors = ErrorList::new();
        assert!(errors.is_empty());

        errors.push("test error".into());
        assert!(!errors.is_empty());
        assert_eq!(errors.len(), 1);

        let error_string = errors.to_string();
        assert!(error_string.contains("test error"));
    }
}
