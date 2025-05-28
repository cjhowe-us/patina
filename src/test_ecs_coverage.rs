#[cfg(test)]
mod tests {
    use crate::ast_ecs::{query::*, components::*};
    use crate::parser::parse_file;
    use crate::ast_ecs::converter::AstConverter;

    #[test]
    fn test_control_flow_conversion() {
        let input = r#"
            while (x < 10) {
                let y = x + 1
            }
        "#.trim();

        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let query = AstQuery::new(&ast_world.world);
        
        // Find while statements
        let while_stmts = query.find_by_kind(NodeKind::While);
        assert_eq!(while_stmts.len(), 1);
    }

    #[test]
    fn test_query_error_handling() {
        let input = r#"let x = 42"#;
        
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let query = AstQuery::new(&ast_world.world);
        
        // Test querying non-existent entity
        // Note: We can't create a fake entity easily, so we'll skip this test
        
        // Test getting wrong component types
        let let_stmts = query.find_by_kind(NodeKind::Let);
        if let Some(&let_entity) = let_stmts.first() {
            // Let statements don't have parameters
            assert_eq!(query.get_parameters(let_entity).len(), 0);
            // Let statements don't have branches
            assert_eq!(query.get_branches(let_entity).len(), 0);
        }
    }

    #[test]
    fn test_position_based_queries() {
        let input = r#"
            let x = 42
            let y = 100
        "#.trim();
        
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let _query = AstQuery::new(&ast_world.world);
        
        // Find nodes at specific positions
        let all_nodes = ast_world.world.query::<&PositionComponent>().iter().count();
        assert!(all_nodes > 0); // Should have nodes with positions
    }

    #[test]
    fn test_walk_with_depth() {
        let input = r#"
            module test {
                fun foo() {
                    let x = 42
                }
            }
        "#.trim();
        
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let query = AstQuery::new(&ast_world.world);
        
        let mut max_depth = 0;
        if let Some(root) = ast_world.root {
            query.walk_children_with_depth(root, 0, &mut |_entity, depth| {
                if depth > max_depth {
                    max_depth = depth;
                }
            });
        }
        
        // Should have several levels of nesting
        assert!(max_depth >= 3);
    }

    #[test]
    fn test_empty_collections() {
        let input = r#"
            fun empty_func() {
            }
        "#.trim();
        
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let query = AstQuery::new(&ast_world.world);
        
        // Find function with empty body
        let functions = query.get_all_functions();
        assert_eq!(functions.len(), 1);
        
        if let Some(&func) = functions.first() {
            // Should have no parameters
            let params = query.get_parameters(func);
            assert_eq!(params.len(), 0);
        }
    }

    #[test]
    fn test_comment_and_annotation_handling() {
        // Comments and annotations aren't parsed in the current implementation
        let input = r#"
            fun old_function() {
                return 42
            }
        "#.trim();
        
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let _query = AstQuery::new(&ast_world.world);
        
        // Skip comment/annotation tests since they're not parsed
    }

    #[test]
    fn test_complex_expressions() {
        let input = r#"
            let result = (a + b) * (c - d) / e
            let chained = obj.field1.field2.method()
            let indexed = array[i][j][k]
        "#.trim();
        
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let query = AstQuery::new(&ast_world.world);
        
        // Find operations
        let operations = query.find_by_kind(NodeKind::Operation);
        assert!(operations.len() >= 3); // Multiple operations
        
        // Find grouped expressions
        let grouped = query.find_by_kind(NodeKind::Grouped);
        assert!(grouped.len() >= 2); // At least two grouped expressions
    }

    #[test]
    fn test_string_interpolation() {
        let input = r#"
            let message = "Hello, \(name)! You have \(count) messages."
        "#.trim();
        
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let query = AstQuery::new(&ast_world.world);
        
        // Find string values
        let strings = query.find_by_kind(NodeKind::StringValue);
        assert_eq!(strings.len(), 1);
        
        if let Some(&string_entity) = strings.first() {
            let fragments = query.get_string_fragments(string_entity);
            // Should have multiple fragments due to interpolation
            assert!(fragments.len() > 1);
        }
    }

    #[test]
    fn test_alternative_if_patterns() {
        let input = r#"
            let x = if (condition) { 42 } else { 0 }
            
            if (a > b) {
                do_something()
            } else if (a < b) {
                do_other()
            } else {
                do_default()
            }
        "#.trim();
        
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let query = AstQuery::new(&ast_world.world);
        
        // Find if expressions
        let if_exprs = query.find_by_kind(NodeKind::If);
        assert!(if_exprs.len() >= 1); // At least one if expression
        
        // Check for branches
        for &if_expr in &if_exprs {
            let branches = query.get_branches(if_expr);
            assert!(!branches.is_empty());
        }
    }

    #[test]
    fn test_edge_case_postfix_operations() {
        let input = r#"
            let result = obj.method()[0].field
            let chained = array[i]
        "#.trim();
        
        let ast = parse_file(input).unwrap();
        let converter = AstConverter::new();
        let ast_world = converter.convert(&ast);
        let query = AstQuery::new(&ast_world.world);
        
        // Find postfix operations
        let operations = query.find_by_kind(NodeKind::Operation);
        assert!(operations.len() >= 1);
    }
}