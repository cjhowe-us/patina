//! Test utilities for AST testing with RON files

use ron;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents a test case with input data and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub input: String,
    pub description: Option<String>,
}

/// Test suite containing multiple test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub test_cases: Vec<TestCase>,
}

/// Load test cases from a RON file
pub fn load_test_suite(filename: &str) -> Result<TestSuite, Box<dyn std::error::Error>> {
    let path = Path::new("test_cases").join(filename);
    let content = fs::read_to_string(&path)?;
    let suite = ron::from_str(&content)?;
    Ok(suite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_ast_printer() {
        use std::fs;
        use std::path::Path;
        
        let dir_path = Path::new("test_cases").join("parser").join("valid");
        
        let ast_test_names = ["static_block", "custom_operators", "function_with_static", "types_as_expressions"];
        
        let mut entries: Vec<_> = fs::read_dir(&dir_path)
            .expect("Failed to read parser test directory")
            .filter_map(|e| e.ok())
            .filter(|e| {
                if let Some(name) = e.path().file_stem().and_then(|s| s.to_str()) {
                    ast_test_names.contains(&name) && 
                    e.path().extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s == "ron")
                        .unwrap_or(false)
                } else {
                    false
                }
            })
            .collect();
        
        // Sort entries for consistent test order
        entries.sort_by_key(|e| e.path());
        
        assert!(!entries.is_empty(), "No test files found in AST test directory");
        
        for entry in entries {
            let path = entry.path();
            
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read test file {:?}: {}", path, e));
            
            let test_case: TestCase = ron::from_str(&content)
                .unwrap_or_else(|e| panic!("Failed to parse RON file {:?}: {}", path, e));
            
            let ast = create_mock_ast(&test_case.name);
            let mut printer = PrinterVisitor::new();
            printer.print_root(&ast.root);
            let output = printer.output;
            
            // Verify the output contains expected elements based on test name
            match test_case.name.as_str() {
                "static_block" => {
                    assert!(output.contains("static {"));
                    assert!(output.contains("}"));
                }
                "custom_operators" => {
                    assert!(output.contains("let result ="));
                    assert!(output.contains("<+>"));
                    assert!(output.contains("<*>"));
                }
                "function_with_static" => {
                    assert!(output.contains("fn compute"));
                    assert!(output.contains("static {"));
                }
                "types_as_expressions" => {
                    assert!(output.contains("let IntType ="));
                }
                _ => {
                    // Just verify we got some output
                    assert!(!output.is_empty());
                }
            }
        }
    }

    fn create_mock_ast(name: &str) -> Ast {
        let mut ast = Ast::new();
        
        match name {
            "static_block" => {
                let static_entity = ast.create_entity();
                let static_stmt = Static {
                    entity: static_entity,
                    statements: vec![],
                };
                ast.root.statements.push(Statement::Static(static_stmt));
            }
            "custom_operators" => {
                let op_entity = ast.create_entity();
                let a = Identifier { entity: ast.create_entity(), name: "a".to_string() };
                let b = Identifier { entity: ast.create_entity(), name: "b".to_string() };
                let c = Identifier { entity: ast.create_entity(), name: "c".to_string() };
                
                let operation = Operation {
                    entity: op_entity,
                    elements: vec![
                        OperationElement::Operand(Expression::Identifier(a)),
                        OperationElement::Operator("<+>".to_string()),
                        OperationElement::Operand(Expression::Identifier(b)),
                        OperationElement::Operator("<*>".to_string()),
                        OperationElement::Operand(Expression::Identifier(c)),
                    ],
                };
                
                let let_entity = ast.create_entity();
                let result_id = Identifier { entity: ast.create_entity(), name: "result".to_string() };
                let let_stmt = Let {
                    entity: let_entity,
                    pattern: Pattern::Identifier(result_id),
                    type_expr: None,
                    value: Some(Expression::Operation(operation)),
                    is_mutable: false,
                    is_static: false,
                };
                ast.root.statements.push(Statement::Let(let_stmt));
            }
            "function_with_static" => {
                let func_entity = ast.create_entity();
                let static_entity = ast.create_entity();
                let static_stmt = Static {
                    entity: static_entity,
                    statements: vec![],
                };
                
                let param_entity = ast.create_entity();
                let param = Parameter {
                    entity: param_entity,
                    name: "x".to_string(),
                    type_expr: Expression::Identifier(Identifier { 
                        entity: ast.create_entity(), 
                        name: "i32".to_string() 
                    }),
                };
                
                let func = Function {
                    entity: func_entity,
                    name: "compute".to_string(),
                    parameters: vec![param],
                    return_type: Some(Expression::Identifier(Identifier { 
                        entity: ast.create_entity(), 
                        name: "i32".to_string() 
                    })),
                    body: Some(Block {
                        entity: ast.create_entity(),
                        label: None,
                        statements: vec![Statement::Static(static_stmt)],
                    }),
                    is_static: false,
                };
                ast.root.statements.push(Statement::Function(func));
            }
            "types_as_expressions" => {
                let let_entity = ast.create_entity();
                let int_type_id = Identifier { entity: ast.create_entity(), name: "IntType".to_string() };
                let i32_id = Identifier { entity: ast.create_entity(), name: "i32".to_string() };
                
                let let_stmt = Let {
                    entity: let_entity,
                    pattern: Pattern::Identifier(int_type_id),
                    type_expr: None,
                    value: Some(Expression::Identifier(i32_id)),
                    is_mutable: false,
                    is_static: false,
                };
                ast.root.statements.push(Statement::Let(let_stmt));
            }
            _ => {}
        }
        
        ast
    }
}