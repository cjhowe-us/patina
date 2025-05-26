use crate::ast::Root;
use crate::test_shared::run_table_tests;
use insta::assert_json_snapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestToken {
    #[serde(rename = "type")]
    token_type: String,
    #[serde(rename = "value")]
    token_value: String,
}

fn parse_patina_code(input: &str) -> Result<Root, String> {
    crate::parser::parse_file(input).map_err(|e| format!("Parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_basic_cases() {
        let test_cases = vec![
            ("Simple binding", "let x = 1"),
            ("Mutable binding", "let mut y = 2"),
            ("Static binding", "static let z = 3"),
            ("Binding with type", "let a: i64 = 4"),
            ("Binding without init", "let b: i64"),
            ("Block atom", "{\n\tlet x = 1\n}"),
            ("Return statement", "return 42"),
            ("Break statement", "break"),
        ];

        run_table_tests("parser_basic", &test_cases, |input, name| {
            let result = parse_patina_code(input);
            assert_json_snapshot!(format!("parser_basic_{}", name.replace(" ", "_")), &result);
            result
        });
    }

    #[test]
    fn test_parser_advanced_cases() {
        let test_cases = vec![
            ("Array type", "[i32; 10]"),
            ("Pointer type", "*mut i32"),
            ("Function type", "fn(i32, i32) -> i32"),
            ("Nested blocks", "{ { let x = 1; x } }"),
            ("String literal", "\"Hello, world!\""),
            ("Integer literal", "42"),
        ];

        run_table_tests("parser_advanced", &test_cases, |input, name| {
            let result = parse_patina_code(input);
            assert_json_snapshot!(format!("parser_advanced_{}", name.replace(" ", "_")), &result);
            result
        });
    }

    #[test]
    fn test_parser_error_cases() {
        let test_cases = vec![
            ("Unclosed block", "{ let x = 1"),
            ("Invalid syntax", "let = 1"),
            ("Unclosed string", "\"hello"),
            ("Invalid type", "let x: *"),
        ];

        run_table_tests("parser_errors", &test_cases, |input, name| {
            let result = parse_patina_code(input);
            // For error cases, we expect failures but still snapshot them
            assert_json_snapshot!(format!("parser_errors_{}", name.replace(" ", "_")), &result);
            result
        });
    }
}
