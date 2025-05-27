use crate::parser::parse_file_with_name;
use crate::test_shared::run_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_from_json() {
        run_tests(
            "TestParser",
            &["parser.json", "parser_additional.json"],
            |input: &String, name: &str| -> Result<crate::ast::Root, Box<dyn std::error::Error>> {
                parse_file_with_name(name, input)
            },
        );
    }

    #[test]
    fn test_parser_errors_from_json() {
        run_tests(
            "TestParserErrors",
            &["parser_error.json", "parser_error_additional.json"],
            |input: &String, name: &str| -> Result<crate::ast::Root, Box<dyn std::error::Error>> {
                parse_file_with_name(name, input)
            },
        );
    }

    #[test]
    fn test_parser_real_world_from_json() {
        run_tests(
            "TestParserRealWorld",
            &["real_world_examples.json"],
            |input: &String, name: &str| -> Result<crate::ast::Root, Box<dyn std::error::Error>> {
                parse_file_with_name(name, input)
            },
        );
    }

    #[test]
    fn test_parser_readme_examples_from_json() {
        run_tests(
            "TestParserReadmeExamples",
            &["readme_examples.json"],
            |input: &String, name: &str| -> Result<crate::ast::Root, Box<dyn std::error::Error>> {
                parse_file_with_name(name, input)
            },
        );
    }

    #[test]
    fn test_while_loops_from_json() {
        run_tests(
            "TestWhileLoops",
            &["while_test.json"],
            |input: &String, name: &str| -> Result<crate::ast::Root, Box<dyn std::error::Error>> {
                parse_file_with_name(name, input)
            },
        );
    }

    #[test]
    fn test_field_access_from_json() {
        run_tests(
            "TestFieldAccess",
            &["field_access_test.json"],
            |input: &String, name: &str| -> Result<crate::ast::Root, Box<dyn std::error::Error>> {
                parse_file_with_name(name, input)
            },
        );
    }

    #[test]
    fn test_lexer_from_json() {
        use crate::parser::{PatinaParser, Rule};
        use pest::Parser;

        run_tests(
            "TestLexer",
            &["lexer.json"],
            |input: &String, _name: &str| -> Result<Vec<String>, Box<dyn std::error::Error>> {
                let pairs = PatinaParser::parse(Rule::root, input)?;
                let tokens: Vec<String> = pairs
                    .flatten()
                    .filter(|p| !matches!(p.as_rule(), Rule::EOI))
                    .map(|p| format!("{:?}: {}", p.as_rule(), p.as_str()))
                    .collect();
                Ok(tokens)
            },
        );
    }
}
