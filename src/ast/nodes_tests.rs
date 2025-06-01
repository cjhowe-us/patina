//! Tests for AST nodes

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use hecs::{Entity, World};

    fn create_test_entity() -> Entity {
        let mut world = World::new();
        world.spawn(())
    }

    #[test]
    fn test_root_node() {
        let entity = create_test_entity();
        let root = Root {
            entity,
            statements: vec![],
        };
        assert_eq!(root.entity_id(), entity);
        assert!(root.statements.is_empty());
    }

    #[test]
    fn test_statement_entity_id() {
        let entity = create_test_entity();
        
        let module = Statement::Module(Module {
            entity,
            name: "test".to_string(),
            statements: vec![],
        });
        assert_eq!(module.entity_id(), entity);
        
        let func = Statement::Function(Function {
            entity,
            name: "foo".to_string(),
            parameters: vec![],
            return_type: None,
            body: None,
            is_static: false,
        });
        assert_eq!(func.entity_id(), entity);
        
        let let_stmt = Statement::Let(Let {
            entity,
            pattern: Pattern::Identifier(Identifier {
                entity: create_test_entity(),
                name: "x".to_string(),
            }),
            type_expr: None,
            value: None,
            is_mutable: false,
            is_static: false,
        });
        assert_eq!(let_stmt.entity_id(), entity);
    }

    #[test]
    fn test_pattern_variants() {
        let entity = create_test_entity();
        
        let id_pattern = Pattern::Identifier(Identifier { entity, name: "x".to_string() });
        assert_eq!(id_pattern.entity_id(), entity);
        
        let tuple_pattern = Pattern::Tuple(TuplePattern {
            entity,
            patterns: vec![],
        });
        assert_eq!(tuple_pattern.entity_id(), entity);
        
        let struct_pattern = Pattern::Struct(StructPattern {
            entity,
            name: "Point".to_string(),
            fields: vec![],
        });
        assert_eq!(struct_pattern.entity_id(), entity);
    }

    #[test]
    fn test_expression_variants() {
        let entity = create_test_entity();
        
        let if_expr = Expression::If(Box::new(If {
            entity,
            branches: vec![],
            else_branch: None,
        }));
        assert_eq!(if_expr.entity_id(), entity);
        
        let op_expr = Expression::Operation(Operation {
            entity,
            elements: vec![],
        });
        assert_eq!(op_expr.entity_id(), entity);
        
        let id_expr = Expression::Identifier(Identifier {
            entity,
            name: "x".to_string(),
        });
        assert_eq!(id_expr.entity_id(), entity);
    }

    #[test]
    fn test_literal_variants() {
        let entity = create_test_entity();
        
        let int_lit = Literal::Integer(IntegerLiteral { entity, value: 42 });
        assert_eq!(int_lit.entity_id(), entity);
        
        let float_lit = Literal::Float(FloatLiteral { entity, value: 3.14 });
        assert_eq!(float_lit.entity_id(), entity);
        
        let bool_lit = Literal::Boolean(BooleanLiteral { entity, value: true });
        assert_eq!(bool_lit.entity_id(), entity);
        
        let null_lit = Literal::Null(NullLiteral { entity });
        assert_eq!(null_lit.entity_id(), entity);
        
        let string_lit = Literal::String(StringLiteral {
            entity,
            fragments: vec![StringFragment::Text("hello".to_string())],
        });
        assert_eq!(string_lit.entity_id(), entity);
    }

    #[test]
    fn test_operation_element() {
        let expr = Expression::Identifier(Identifier {
            entity: create_test_entity(),
            name: "x".to_string(),
        });
        
        let operand = OperationElement::Operand(expr);
        let operator = OperationElement::Operator("+".to_string());
        
        match operand {
            OperationElement::Operand(_) => {},
            _ => panic!("Expected operand"),
        }
        
        match operator {
            OperationElement::Operator(op) => assert_eq!(op, "+"),
            _ => panic!("Expected operator"),
        }
    }

    #[test]
    fn test_string_fragment() {
        let text_frag = StringFragment::Text("hello".to_string());
        let interp_frag = StringFragment::Interpolation(
            Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "x".to_string(),
            })
        );
        
        match text_frag {
            StringFragment::Text(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected text fragment"),
        }
        
        match interp_frag {
            StringFragment::Interpolation(_) => {},
            _ => panic!("Expected interpolation fragment"),
        }
    }

    #[test]
    fn test_complex_structures() {
        let entity = create_test_entity();
        
        let lambda = Lambda {
            entity,
            parameters: vec![
                Parameter {
                    entity: create_test_entity(),
                    name: "x".to_string(),
                    type_expr: Expression::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "i32".to_string(),
                    }),
                }
            ],
            return_type: Some(Box::new(Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "i32".to_string(),
            }))),
            body: Box::new(Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "x".to_string(),
            })),
        };
        
        assert_eq!(lambda.parameters.len(), 1);
        assert!(lambda.return_type.is_some());
    }

    #[test]
    fn test_static_statement() {
        let entity = create_test_entity();
        let static_stmt = Static {
            entity,
            statements: vec![
                Statement::Let(Let {
                    entity: create_test_entity(),
                    pattern: Pattern::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "x".to_string(),
                    }),
                    type_expr: None,
                    value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                        entity: create_test_entity(),
                        value: 42,
                    }))),
                    is_mutable: false,
                    is_static: false,
                })
            ],
        };
        
        assert_eq!(static_stmt.statements.len(), 1);
    }
}