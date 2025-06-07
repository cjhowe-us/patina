//! Additional tests for AST nodes to improve coverage

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use hecs::{Entity, World};

    fn create_test_entity() -> Entity {
        let mut world = World::new();
        world.spawn(())
    }

    #[test]
    fn test_all_statement_types() {
        let entity = create_test_entity();

        // Test Module
        let module = Module {
            entity,
            name: "test_module".to_string(),
            statements: vec![],
        };
        assert_eq!(module.entity, entity);
        assert_eq!(module.name, "test_module");

        // Test Function
        let func = Function {
            entity,
            name: "test_func".to_string(),
            parameters: vec![],
            return_type: None,
            body: None,
            is_static: true,
        };
        assert!(func.is_static);
        assert!(func.body.is_none());

        // Test Return
        let ret = Return {
            entity,
            label: Some("loop1".to_string()),
            value: None,
        };
        assert_eq!(ret.label, Some("loop1".to_string()));

        // Test While
        let while_stmt = While {
            entity,
            label: Some("outer".to_string()),
            condition: Expression::Literal(Literal::Boolean(BooleanLiteral {
                entity: create_test_entity(),
                value: true,
            })),
            body: Block {
                entity: create_test_entity(),
                label: None,
                statements: vec![],
            },
        };
        assert_eq!(while_stmt.label, Some("outer".to_string()));

        // Test For
        let for_stmt = For {
            entity,
            label: None,
            init: Some(Box::new(Statement::Expression(Expression::Literal(
                Literal::Integer(IntegerLiteral {
                    entity: create_test_entity(),
                    value: 0,
                }),
            )))),
            condition: None,
            update: None,
            body: Block {
                entity: create_test_entity(),
                label: None,
                statements: vec![],
            },
        };
        assert!(for_stmt.init.is_some());

        // Test Break
        let break_stmt = Break {
            entity,
            label: None,
            value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                entity: create_test_entity(),
                value: 42,
            }))),
        };
        assert!(break_stmt.value.is_some());

        // Test Block
        let block = Block {
            entity,
            label: Some("named".to_string()),
            statements: vec![],
        };
        assert_eq!(block.label, Some("named".to_string()));
    }

    #[test]
    fn test_all_expression_types() {
        let entity = create_test_entity();

        // Test Case
        let case = Case {
            entity,
            value: Box::new(Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "x".to_string(),
            })),
            branches: vec![],
        };
        assert_eq!(
            Statement::Expression(Expression::Case(Box::new(case.clone()))).entity_id(),
            entity
        );

        // Test Call
        let call = Call {
            entity,
            function: Box::new(Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "func".to_string(),
            })),
            arguments: vec![],
        };
        assert_eq!(Expression::Call(call).entity_id(), entity);

        // Test Index
        let index = Index {
            entity,
            object: Box::new(Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "arr".to_string(),
            })),
            index: Box::new(Expression::Literal(Literal::Integer(IntegerLiteral {
                entity: create_test_entity(),
                value: 0,
            }))),
        };
        assert_eq!(Expression::Index(index).entity_id(), entity);

        // Test Field
        let field = Field {
            entity,
            object: Box::new(Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "obj".to_string(),
            })),
            field: "field_name".to_string(),
        };
        assert_eq!(field.field, "field_name");
        assert_eq!(Expression::Field(field).entity_id(), entity);
    }

    #[test]
    fn test_if_and_case_branches() {
        let entity = create_test_entity();

        // Test IfBranch
        let if_branch = IfBranch {
            entity,
            condition: Expression::Literal(Literal::Boolean(BooleanLiteral {
                entity: create_test_entity(),
                value: true,
            })),
            body: Expression::Literal(Literal::Integer(IntegerLiteral {
                entity: create_test_entity(),
                value: 1,
            })),
        };
        assert_eq!(if_branch.entity_id(), entity);

        // Test CaseBranch with guard
        let case_branch = CaseBranch {
            entity,
            pattern: Pattern::Identifier(Identifier {
                entity: create_test_entity(),
                name: "Some".to_string(),
            }),
            guard: Some(Expression::Literal(Literal::Boolean(BooleanLiteral {
                entity: create_test_entity(),
                value: true,
            }))),
            body: Expression::Literal(Literal::Integer(IntegerLiteral {
                entity: create_test_entity(),
                value: 42,
            })),
        };
        assert_eq!(case_branch.entity_id(), entity);
        assert!(case_branch.guard.is_some());
    }

    #[test]
    fn test_pattern_destructuring() {
        let entity = create_test_entity();

        // Test TuplePattern with multiple patterns
        let tuple_pattern = TuplePattern {
            entity,
            patterns: vec![
                Pattern::Identifier(Identifier {
                    entity: create_test_entity(),
                    name: "x".to_string(),
                }),
                Pattern::Identifier(Identifier {
                    entity: create_test_entity(),
                    name: "y".to_string(),
                }),
            ],
        };
        assert_eq!(tuple_pattern.patterns.len(), 2);

        // Test StructPattern with fields
        let struct_pattern = StructPattern {
            entity,
            name: "Point".to_string(),
            fields: vec![
                (
                    "x".to_string(),
                    Pattern::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "px".to_string(),
                    }),
                ),
                (
                    "y".to_string(),
                    Pattern::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "py".to_string(),
                    }),
                ),
            ],
        };
        assert_eq!(struct_pattern.fields.len(), 2);
        assert_eq!(struct_pattern.fields[0].0, "x");
    }

    #[test]
    fn test_parameter_and_lambda() {
        let entity = create_test_entity();

        // Test Parameter
        let param = Parameter {
            entity,
            name: "value".to_string(),
            type_expr: Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "String".to_string(),
            }),
        };
        assert_eq!(param.entity_id(), entity);
        assert_eq!(param.name, "value");

        // Test Lambda with multiple parameters and return type
        let lambda = Lambda {
            entity,
            parameters: vec![
                Parameter {
                    entity: create_test_entity(),
                    name: "a".to_string(),
                    type_expr: Expression::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "i32".to_string(),
                    }),
                },
                Parameter {
                    entity: create_test_entity(),
                    name: "b".to_string(),
                    type_expr: Expression::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "i32".to_string(),
                    }),
                },
            ],
            return_type: Some(Box::new(Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "i32".to_string(),
            }))),
            body: Box::new(Expression::Operation(Operation {
                entity: create_test_entity(),
                elements: vec![
                    OperationElement::Operand(Expression::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "a".to_string(),
                    })),
                    OperationElement::Operator("+".to_string()),
                    OperationElement::Operand(Expression::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "b".to_string(),
                    })),
                ],
            })),
        };
        assert_eq!(lambda.parameters.len(), 2);
        assert!(lambda.return_type.is_some());
    }

    #[test]
    fn test_let_statement_variations() {
        let entity = create_test_entity();

        // Test static mutable let
        let static_mut_let = Let {
            entity,
            pattern: Pattern::Identifier(Identifier {
                entity: create_test_entity(),
                name: "COUNTER".to_string(),
            }),
            type_expr: Some(Expression::Identifier(Identifier {
                entity: create_test_entity(),
                name: "u64".to_string(),
            })),
            value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                entity: create_test_entity(),
                value: 0,
            }))),
            is_mutable: true,
            is_static: true,
        };
        assert!(static_mut_let.is_mutable);
        assert!(static_mut_let.is_static);

        // Test destructuring let
        let destructure_let = Let {
            entity,
            pattern: Pattern::Tuple(TuplePattern {
                entity: create_test_entity(),
                patterns: vec![
                    Pattern::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "x".to_string(),
                    }),
                    Pattern::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "y".to_string(),
                    }),
                ],
            }),
            type_expr: None,
            value: Some(Expression::Call(Call {
                entity: create_test_entity(),
                function: Box::new(Expression::Identifier(Identifier {
                    entity: create_test_entity(),
                    name: "get_point".to_string(),
                })),
                arguments: vec![],
            })),
            is_mutable: false,
            is_static: false,
        };

        match &destructure_let.pattern {
            Pattern::Tuple(t) => assert_eq!(t.patterns.len(), 2),
            _ => panic!("Expected tuple pattern"),
        }
    }

    #[test]
    fn test_expression_block() {
        let entity = create_test_entity();

        let block = Block {
            entity,
            label: None,
            statements: vec![
                Statement::Let(Let {
                    entity: create_test_entity(),
                    pattern: Pattern::Identifier(Identifier {
                        entity: create_test_entity(),
                        name: "temp".to_string(),
                    }),
                    type_expr: None,
                    value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                        entity: create_test_entity(),
                        value: 5,
                    }))),
                    is_mutable: false,
                    is_static: false,
                }),
                Statement::Expression(Expression::Operation(Operation {
                    entity: create_test_entity(),
                    elements: vec![
                        OperationElement::Operand(Expression::Identifier(Identifier {
                            entity: create_test_entity(),
                            name: "temp".to_string(),
                        })),
                        OperationElement::Operator("*".to_string()),
                        OperationElement::Operand(Expression::Literal(Literal::Integer(
                            IntegerLiteral {
                                entity: create_test_entity(),
                                value: 2,
                            },
                        ))),
                    ],
                })),
            ],
        };

        let block_expr = Expression::Block(block);
        assert_eq!(block_expr.entity_id(), entity);
    }

    #[test]
    fn test_float_literal() {
        let entity = create_test_entity();

        let float_lit = FloatLiteral {
            entity,
            value: std::f64::consts::PI,
        };
        assert_eq!(float_lit.value, std::f64::consts::PI);

        let lit = Literal::Float(float_lit);
        assert_eq!(lit.entity_id(), entity);
    }

    #[test]
    fn test_complex_string_literal() {
        let entity = create_test_entity();

        let string_lit = StringLiteral {
            entity,
            fragments: vec![
                StringFragment::Text("The answer is ".to_string()),
                StringFragment::Interpolation(Expression::Operation(Operation {
                    entity: create_test_entity(),
                    elements: vec![
                        OperationElement::Operand(Expression::Literal(Literal::Integer(
                            IntegerLiteral {
                                entity: create_test_entity(),
                                value: 21,
                            },
                        ))),
                        OperationElement::Operator("*".to_string()),
                        OperationElement::Operand(Expression::Literal(Literal::Integer(
                            IntegerLiteral {
                                entity: create_test_entity(),
                                value: 2,
                            },
                        ))),
                    ],
                })),
                StringFragment::Text(" and the value is ".to_string()),
                StringFragment::Interpolation(Expression::Identifier(Identifier {
                    entity: create_test_entity(),
                    name: "x".to_string(),
                })),
                StringFragment::Text(".".to_string()),
            ],
        };

        assert_eq!(string_lit.fragments.len(), 5);
        let lit = Literal::String(string_lit);
        assert_eq!(lit.entity_id(), entity);
    }

    #[test]
    fn test_grouped_expression() {
        let _entity = create_test_entity();

        // Create a complex grouped expression
        let inner_op = Expression::Operation(Operation {
            entity: create_test_entity(),
            elements: vec![
                OperationElement::Operand(Expression::Identifier(Identifier {
                    entity: create_test_entity(),
                    name: "a".to_string(),
                })),
                OperationElement::Operator("+".to_string()),
                OperationElement::Operand(Expression::Identifier(Identifier {
                    entity: create_test_entity(),
                    name: "b".to_string(),
                })),
            ],
        });

        let grouped = Expression::Grouped(Box::new(inner_op));

        // Test that grouped expressions preserve the inner expression's entity
        match grouped {
            Expression::Grouped(inner) => match inner.as_ref() {
                Expression::Operation(op) => {
                    assert!(!op.elements.is_empty());
                }
                _ => panic!("Expected operation"),
            },
            _ => panic!("Expected grouped expression"),
        }
    }
}
