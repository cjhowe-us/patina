use crate::ast::SyntaxNode;
use crate::parser::parse_file_with_name;
use crate::test_shared::run_tests;
use crate::visitor::{walk, DebugPrinter, NodeCounter, ValidationVisitor, Visitor};
use insta::assert_json_snapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct VisitLog {
    #[serde(rename = "type")]
    node_type: String,
    message: String,
    is_enter: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct LogVisitor {
    log: Vec<VisitLog>,
}

impl LogVisitor {
    fn new() -> Self {
        Self { log: Vec::new() }
    }
}

impl Visitor<(), String> for LogVisitor {
    fn enter(
        &mut self,
        node: &dyn SyntaxNode,
        _request: Option<&()>,
    ) -> Result<Vec<Option<()>>, Box<dyn std::error::Error>> {
        // Log entry
        self.log.push(VisitLog {
            node_type: std::any::type_name_of_val(node).to_string(),
            message: format!("Entering node at position {:?}", node.position()),
            is_enter: true,
        });

        // Return None for each child (no specific input needed)
        let children_count = node.children().len();
        Ok(vec![None; children_count])
    }

    fn exit(
        &mut self,
        node: &dyn SyntaxNode,
        _children_outputs: Vec<Option<String>>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Log exit
        self.log.push(VisitLog {
            node_type: std::any::type_name_of_val(node).to_string(),
            message: format!("Exiting node at position {:?}", node.position()),
            is_enter: false,
        });

        Ok(Some(format!(
            "Visited: {}",
            std::any::type_name_of_val(node)
        )))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WalkTestOutput {
    result: Option<String>,
    visitor: LogVisitor,
}

fn parse_and_walk(input: &str) -> Result<WalkTestOutput, String> {
    let root = crate::parser::parse_file_with_name("test_input", input)
        .map_err(|e| format!("Parse error: {}", e))?;

    let mut visitor = LogVisitor::new();
    let result = walk(&mut visitor, None, &root).map_err(|e| format!("Visitor error: {}", e))?;

    Ok(WalkTestOutput { result, visitor })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WalkOutput {
    node_types: Vec<String>,
    visit_count: HashMap<String, usize>,
}

struct WalkCollector {
    node_types: Vec<String>,
    visit_count: HashMap<String, usize>,
}

impl WalkCollector {
    fn new() -> Self {
        Self {
            node_types: Vec::new(),
            visit_count: HashMap::new(),
        }
    }
}

impl Visitor<(), ()> for WalkCollector {
    fn enter(
        &mut self,
        node: &dyn crate::ast::SyntaxNode,
        _request: Option<&()>,
    ) -> Result<Vec<Option<()>>, Box<dyn std::error::Error>> {
        let node_type = get_node_type_name(node);
        self.node_types.push(node_type.clone());
        *self.visit_count.entry(node_type).or_insert(0) += 1;

        let children_count = node.children().len();
        Ok(vec![None; children_count])
    }

    fn exit(
        &mut self,
        _node: &dyn crate::ast::SyntaxNode,
        _children_outputs: Vec<Option<()>>,
    ) -> Result<Option<()>, Box<dyn std::error::Error>> {
        Ok(None)
    }
}

// Helper function to get a clean type name
fn get_node_type_name(node: &dyn crate::ast::SyntaxNode) -> String {
    let full_name = std::any::type_name_of_val(node);
    // Extract just the type name without the module path
    full_name
        .split("::")
        .last()
        .unwrap_or(full_name)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor_basic_cases() {
        // Simple binding
        let result = parse_and_walk("let x = 1");
        assert_json_snapshot!("visitor_basic_Simple_binding", &result);

        // Block expression
        let result = parse_and_walk("{ let x = 1; x }");
        assert_json_snapshot!("visitor_basic_Block_expression", &result);

        // Return statement
        let result = parse_and_walk("return 42");
        assert_json_snapshot!("visitor_basic_Return_statement", &result);
    }

    #[test]
    fn test_visitor_complex_cases() {
        // Nested expressions
        let result = parse_and_walk("{ { let x = 1 } }");
        assert_json_snapshot!("visitor_complex_Nested_expressions", &result);

        // Type annotations
        let result = parse_and_walk("let x: i32 = 42");
        assert_json_snapshot!("visitor_complex_Type_annotations", &result);

        // Array type - expected to fail with a parse error
        let result = parse_and_walk("[i32; 10]");
        assert_json_snapshot!("visitor_complex_Array_type", &result);
    }

    #[test]
    fn test_debug_printer() {
        let input = "let x = 42";
        let root = crate::parser::parse_file_with_name("debug_printer_test", input).unwrap();

        let mut printer = DebugPrinter::new();
        let result = walk(&mut printer, None, &root);

        // Convert result to a serializable format
        let output = match result {
            Ok(value) => format!("Success: {:?}", value),
            Err(e) => format!("Error: {}", e),
        };

        assert_json_snapshot!("debug_printer_output", &output);
    }

    #[test]
    fn test_validation_visitor() {
        let input = "let x = 42";
        let root = crate::parser::parse_file_with_name("validation_test", input).unwrap();

        let mut validator = ValidationVisitor::new();
        let result = walk(&mut validator, None, &root);

        // Convert result to a serializable format
        let output = match result {
            Ok(value) => format!("Success: {:?}", value),
            Err(e) => format!("Error: {}", e),
        };

        assert_json_snapshot!("validation_visitor_output", &output);
    }

    #[test]
    fn test_node_counter() {
        let input = "{ let x = 1; let y = 2; x + y }";
        let root = crate::parser::parse_file_with_name("node_counter_test", input).unwrap();

        let mut counter = NodeCounter::new();
        let result = walk(&mut counter, None, &root);

        // Convert result to a serializable format
        let output = match result {
            Ok(value) => format!("Success: {:?}", value),
            Err(e) => format!("Error: {}", e),
        };

        assert_json_snapshot!("node_counter_output", &output);
    }

    #[test]
    fn test_walk_from_json() {
        run_tests(
            "TestWalk",
            &["walk.json"],
            |input: &String, name: &str| -> Result<WalkOutput, Box<dyn std::error::Error>> {
                let root = parse_file_with_name(name, input)?;

                let mut collector = WalkCollector::new();
                walk(&mut collector, None, &root)?;

                Ok(WalkOutput {
                    node_types: collector.node_types,
                    visit_count: collector.visit_count,
                })
            },
        );
    }
}
