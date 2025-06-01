use crate::parser::parse_file_with_name;
use crate::test_shared::{run_parser_directory_tests, run_directory_tests};
use crate::ast::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_valid() {
        run_parser_directory_tests(
            "Parser",
            "parser/valid",
            |input: &str, name: &str| -> Result<crate::ast::Ast, Box<dyn std::error::Error>> {
                parse_file_with_name(name, input)
            },
            |name: &str, ast: &crate::ast::Ast| {
                // Basic verification - just ensure we got a valid AST
                // Specific tests can be added based on test name if needed
                match name {
                    "Module" => {
                        assert_eq!(ast.root.statements.len(), 1);
                        match &ast.root.statements[0] {
                            Statement::Module(m) => {
                                assert_eq!(m.name, "test");
                                assert_eq!(m.statements.len(), 1);
                            }
                            _ => panic!("Expected Module statement"),
                        }
                    }
                    "Simple binding" => {
                        assert_eq!(ast.root.statements.len(), 1);
                        match &ast.root.statements[0] {
                            Statement::Let(l) => {
                                assert!(!l.is_mutable);
                                assert!(!l.is_static);
                                match &l.pattern {
                                    Pattern::Identifier(id) => assert_eq!(id.name, "x"),
                                    _ => panic!("Expected identifier pattern"),
                                }
                            }
                            _ => panic!("Expected Let statement"),
                        }
                    }
                    "Mutable binding" => {
                        assert_eq!(ast.root.statements.len(), 1);
                        match &ast.root.statements[0] {
                            Statement::Let(l) => {
                                assert!(l.is_mutable);
                                assert!(!l.is_static);
                                match &l.pattern {
                                    Pattern::Identifier(id) => assert_eq!(id.name, "y"),
                                    _ => panic!("Expected identifier pattern"),
                                }
                            }
                            _ => panic!("Expected Let statement"),
                        }
                    }
                    "Static binding" => {
                        assert_eq!(ast.root.statements.len(), 1);
                        match &ast.root.statements[0] {
                            Statement::Let(l) => {
                                assert!(!l.is_mutable);
                                assert!(l.is_static);
                                match &l.pattern {
                                    Pattern::Identifier(id) => assert_eq!(id.name, "z"),
                                    _ => panic!("Expected identifier pattern"),
                                }
                            }
                            _ => panic!("Expected Let statement"),
                        }
                    }
                    _ => {
                        // For other tests, just verify it parsed successfully
                        // The AST structure exists and has the root node
                        assert!(ast.world.len() > 0);
                    }
                }
            },
        );
    }

    #[test]
    fn test_parser_errors() {
        run_directory_tests(
            "ParserErrors",
            "parser/errors",
            |input: &str, name: &str| -> Result<crate::ast::Ast, Box<dyn std::error::Error>> {
                parse_file_with_name(name, input)
            },
        );
    }

}