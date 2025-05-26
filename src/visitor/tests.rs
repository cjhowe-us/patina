use crate::ast::SyntaxNode;
use crate::test_shared::run_table_tests;
use crate::visitor::{Visitor, DebugPrinter, ValidationVisitor, NodeCounter, walk};
use insta::assert_json_snapshot;
use serde::{Deserialize, Serialize};

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
    fn enter(&mut self, node: &dyn SyntaxNode, _request: Option<&()>) -> Result<Vec<Option<()>>, Box<dyn std::error::Error>> {
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

    fn exit(&mut self, node: &dyn SyntaxNode, _children_outputs: Vec<Option<String>>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Log exit
        self.log.push(VisitLog {
            node_type: std::any::type_name_of_val(node).to_string(),
            message: format!("Exiting node at position {:?}", node.position()),
            is_enter: false,
        });

        Ok(Some(format!("Visited: {}", std::any::type_name_of_val(node))))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WalkTestOutput {
    result: Option<String>,
    visitor: LogVisitor,
}

fn parse_and_walk(input: &str) -> Result<WalkTestOutput, String> {
    let root = crate::parser::parse_file(input).map_err(|e| format!("Parse error: {}", e))?;

    let mut visitor = LogVisitor::new();
    let result = walk(&mut visitor, None, &root)
        .map_err(|e| format!("Visitor error: {}", e))?;

    Ok(WalkTestOutput {
        result,
        visitor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor_basic_cases() {
        let test_cases = vec![
            ("Simple binding", "let x = 1"),
            ("Block expression", "{ let x = 1; x }"),
            ("Return statement", "return 42"),
        ];

        run_table_tests("visitor_basic", &test_cases, |input, name| {
            let result = parse_and_walk(input);
            assert_json_snapshot!(format!("visitor_basic_{}", name.replace(" ", "_")), &result);
            result
        });
    }

    #[test]
    fn test_visitor_complex_cases() {
        let test_cases = vec![
            ("Nested expressions", "{ { let x = 1 } }"),
            ("Type annotations", "let x: i32 = 42"),
            ("Array type", "[i32; 10]"),
        ];

        run_table_tests("visitor_complex", &test_cases, |input, name| {
            let result = parse_and_walk(input);
            assert_json_snapshot!(
                format!("visitor_complex_{}", name.replace(" ", "_")),
                &result
            );
            result
        });
    }

    #[test]
    fn test_debug_printer() {
        let input = "let x = 42";
        let root = crate::parser::parse_file(input).unwrap();

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
        let root = crate::parser::parse_file(input).unwrap();

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
        let root = crate::parser::parse_file(input).unwrap();

        let mut counter = NodeCounter::new();
        let result = walk(&mut counter, None, &root);

        // Convert result to a serializable format
        let output = match result {
            Ok(value) => format!("Success: {:?}", value),
            Err(e) => format!("Error: {}", e),
        };

        assert_json_snapshot!("node_counter_output", &output);
    }
}
