//! Additional tests for the AST printer to improve coverage

#[cfg(test)]
mod tests {
    use crate::ast::*;

    fn print_ast(ast: &Ast) -> String {
        let mut printer = PrinterVisitor::new();
        printer.print_root(&ast.root);
        printer.output
    }

    fn create_test_ast() -> Ast {
        Ast::new()
    }

    #[test]
    fn test_print_return_with_label() {
        let mut ast = create_test_ast();

        let ret = Statement::Return(Return {
            entity: ast.create_entity(),
            label: Some("outer_loop".to_string()),
            value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                entity: ast.create_entity(),
                value: 42,
            }))),
        });

        ast.root.statements.push(ret);

        let output = print_ast(&ast);
        assert_eq!(output, "return 'outer_loop 42;\n");
    }

    #[test]
    fn test_print_while_with_label() {
        let mut ast = create_test_ast();

        let while_stmt = Statement::While(While {
            entity: ast.create_entity(),
            label: Some("main_loop".to_string()),
            condition: Expression::Literal(Literal::Boolean(BooleanLiteral {
                entity: ast.create_entity(),
                value: true,
            })),
            body: Block {
                entity: ast.create_entity(),
                label: None,
                statements: vec![],
            },
        });

        ast.root.statements.push(while_stmt);

        let output = print_ast(&ast);
        assert_eq!(output, "'main_loop: while true {\n}\n");
    }

    #[test]
    fn test_print_for_loop() {
        let mut ast = create_test_ast();

        let for_stmt = Statement::For(For {
            entity: ast.create_entity(),
            label: None,
            init: Some(Box::new(Statement::Let(Let {
                entity: ast.create_entity(),
                pattern: Pattern::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "i".to_string(),
                }),
                type_expr: None,
                value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                    entity: ast.create_entity(),
                    value: 0,
                }))),
                is_mutable: true,
                is_static: false,
            }))),
            condition: Some(Expression::Operation(Operation {
                entity: ast.create_entity(),
                elements: vec![
                    OperationElement::Operand(Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "i".to_string(),
                    })),
                    OperationElement::Operator("<".to_string()),
                    OperationElement::Operand(Expression::Literal(Literal::Integer(
                        IntegerLiteral {
                            entity: ast.create_entity(),
                            value: 10,
                        },
                    ))),
                ],
            })),
            update: Some(Expression::Operation(Operation {
                entity: ast.create_entity(),
                elements: vec![
                    OperationElement::Operand(Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "i".to_string(),
                    })),
                    OperationElement::Operator("+=".to_string()),
                    OperationElement::Operand(Expression::Literal(Literal::Integer(
                        IntegerLiteral {
                            entity: ast.create_entity(),
                            value: 1,
                        },
                    ))),
                ],
            })),
            body: Block {
                entity: ast.create_entity(),
                label: None,
                statements: vec![],
            },
        });

        ast.root.statements.push(for_stmt);

        let output = print_ast(&ast);
        assert_eq!(output, "for (mut let i = 0; i < 10; i += 1) {\n}\n");
    }

    #[test]
    fn test_print_break_with_label_and_value() {
        let mut ast = create_test_ast();

        let break_stmt = Statement::Break(Break {
            entity: ast.create_entity(),
            label: Some("outer".to_string()),
            value: Some(Expression::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "result".to_string(),
            })),
        });

        ast.root.statements.push(break_stmt);

        let output = print_ast(&ast);
        assert_eq!(output, "break 'outer result;\n");
    }

    #[test]
    fn test_print_block_with_label() {
        let mut ast = create_test_ast();

        let block = Statement::Block(Block {
            entity: ast.create_entity(),
            label: Some("my_block".to_string()),
            statements: vec![Statement::Expression(Expression::Literal(
                Literal::Integer(IntegerLiteral {
                    entity: ast.create_entity(),
                    value: 42,
                }),
            ))],
        });

        ast.root.statements.push(block);

        let output = print_ast(&ast);
        assert_eq!(output, "'my_block: {\n  42\n}\n");
    }

    #[test]
    fn test_print_case_expression() {
        let mut ast = create_test_ast();

        let case_expr = Expression::Case(Box::new(Case {
            entity: ast.create_entity(),
            value: Box::new(Expression::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "x".to_string(),
            })),
            branches: vec![
                CaseBranch {
                    entity: ast.create_entity(),
                    pattern: Pattern::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "Some".to_string(),
                    }),
                    guard: Some(Expression::Operation(Operation {
                        entity: ast.create_entity(),
                        elements: vec![
                            OperationElement::Operand(Expression::Identifier(Identifier {
                                entity: ast.create_entity(),
                                name: "val".to_string(),
                            })),
                            OperationElement::Operator(">".to_string()),
                            OperationElement::Operand(Expression::Literal(Literal::Integer(
                                IntegerLiteral {
                                    entity: ast.create_entity(),
                                    value: 0,
                                },
                            ))),
                        ],
                    })),
                    body: Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "val".to_string(),
                    }),
                },
                CaseBranch {
                    entity: ast.create_entity(),
                    pattern: Pattern::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "_".to_string(),
                    }),
                    guard: None,
                    body: Expression::Literal(Literal::Integer(IntegerLiteral {
                        entity: ast.create_entity(),
                        value: -1,
                    })),
                },
            ],
        }));

        ast.root.statements.push(Statement::Expression(case_expr));

        let output = print_ast(&ast);
        assert!(output.contains("case x {"));
        assert!(output.contains("Some if val > 0 => val,"));
        assert!(output.contains("_ => -1,"));
    }

    #[test]
    fn test_print_call_and_index() {
        let mut ast = create_test_ast();

        let expr = Expression::Call(Call {
            entity: ast.create_entity(),
            function: Box::new(Expression::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "func".to_string(),
            })),
            arguments: vec![
                Expression::Index(Index {
                    entity: ast.create_entity(),
                    object: Box::new(Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "arr".to_string(),
                    })),
                    index: Box::new(Expression::Literal(Literal::Integer(IntegerLiteral {
                        entity: ast.create_entity(),
                        value: 0,
                    }))),
                }),
                Expression::Field(Field {
                    entity: ast.create_entity(),
                    object: Box::new(Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "obj".to_string(),
                    })),
                    field: "value".to_string(),
                }),
            ],
        });

        ast.root.statements.push(Statement::Expression(expr));

        let output = print_ast(&ast);
        assert_eq!(output, "func(arr[0], obj.value)\n");
    }

    #[test]
    fn test_print_literals() {
        let mut ast = create_test_ast();

        // Float literal
        let float_entity = ast.create_entity();
        ast.root
            .statements
            .push(Statement::Expression(Expression::Literal(Literal::Float(
                FloatLiteral {
                    entity: float_entity,
                    value: 3.14159,
                },
            ))));

        // Boolean literals
        let true_entity = ast.create_entity();
        ast.root
            .statements
            .push(Statement::Expression(Expression::Literal(
                Literal::Boolean(BooleanLiteral {
                    entity: true_entity,
                    value: true,
                }),
            )));

        let false_entity = ast.create_entity();
        ast.root
            .statements
            .push(Statement::Expression(Expression::Literal(
                Literal::Boolean(BooleanLiteral {
                    entity: false_entity,
                    value: false,
                }),
            )));

        // Null literal
        let null_entity = ast.create_entity();
        ast.root
            .statements
            .push(Statement::Expression(Expression::Literal(Literal::Null(
                NullLiteral {
                    entity: null_entity,
                },
            ))));

        let output = print_ast(&ast);
        assert!(output.contains("3.14159"));
        assert!(output.contains("true"));
        assert!(output.contains("false"));
        assert!(output.contains("null"));
    }

    #[test]
    fn test_print_pattern_variations() {
        let mut ast = create_test_ast();

        // Tuple pattern
        let tuple_let = Statement::Let(Let {
            entity: ast.create_entity(),
            pattern: Pattern::Tuple(TuplePattern {
                entity: ast.create_entity(),
                patterns: vec![
                    Pattern::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "x".to_string(),
                    }),
                    Pattern::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "y".to_string(),
                    }),
                ],
            }),
            type_expr: None,
            value: None,
            is_mutable: false,
            is_static: false,
        });

        // Struct pattern
        let struct_let = Statement::Let(Let {
            entity: ast.create_entity(),
            pattern: Pattern::Struct(StructPattern {
                entity: ast.create_entity(),
                name: "Point".to_string(),
                fields: vec![
                    (
                        "x".to_string(),
                        Pattern::Identifier(Identifier {
                            entity: ast.create_entity(),
                            name: "px".to_string(),
                        }),
                    ),
                    (
                        "y".to_string(),
                        Pattern::Identifier(Identifier {
                            entity: ast.create_entity(),
                            name: "py".to_string(),
                        }),
                    ),
                ],
            }),
            type_expr: None,
            value: None,
            is_mutable: false,
            is_static: false,
        });

        ast.root.statements.push(tuple_let);
        ast.root.statements.push(struct_let);

        let output = print_ast(&ast);
        assert!(output.contains("let (x, y);"));
        assert!(output.contains("let Point { x: px, y: py };"));
    }

    #[test]
    fn test_print_static_let() {
        let mut ast = create_test_ast();

        let static_let = Statement::Let(Let {
            entity: ast.create_entity(),
            pattern: Pattern::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "CONSTANT".to_string(),
            }),
            type_expr: Some(Expression::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "u32".to_string(),
            })),
            value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                entity: ast.create_entity(),
                value: 100,
            }))),
            is_mutable: false,
            is_static: true,
        });

        ast.root.statements.push(static_let);

        let output = print_ast(&ast);
        assert_eq!(output, "static let CONSTANT: u32 = 100;\n");
    }

    #[test]
    fn test_print_function_no_body() {
        let mut ast = create_test_ast();

        let func = Statement::Function(Function {
            entity: ast.create_entity(),
            name: "external".to_string(),
            parameters: vec![Parameter {
                entity: ast.create_entity(),
                name: "x".to_string(),
                type_expr: Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "i32".to_string(),
                }),
            }],
            return_type: Some(Expression::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "i32".to_string(),
            })),
            body: None,
            is_static: true,
        });

        ast.root.statements.push(func);

        let output = print_ast(&ast);
        assert_eq!(output, "static fn external(x: i32): i32;\n");
    }

    #[test]
    fn test_print_string_escapes() {
        let mut ast = create_test_ast();

        let string_lit = Expression::Literal(Literal::String(StringLiteral {
            entity: ast.create_entity(),
            fragments: vec![StringFragment::Text(
                "Line 1\nLine 2\tTabbed\r\n\"Quoted\" and \\backslash\\".to_string(),
            )],
        }));

        ast.root.statements.push(Statement::Expression(string_lit));

        let output = print_ast(&ast);
        assert!(output.contains("\\n"));
        assert!(output.contains("\\t"));
        assert!(output.contains("\\r"));
        assert!(output.contains("\\\""));
        assert!(output.contains("\\\\"));
    }

    #[test]
    fn test_print_grouped_expression() {
        let mut ast = create_test_ast();

        let grouped = Expression::Grouped(Box::new(Expression::Operation(Operation {
            entity: ast.create_entity(),
            elements: vec![
                OperationElement::Operand(Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "a".to_string(),
                })),
                OperationElement::Operator("+".to_string()),
                OperationElement::Operand(Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "b".to_string(),
                })),
            ],
        })));

        ast.root.statements.push(Statement::Expression(grouped));

        let output = print_ast(&ast);
        assert_eq!(output, "(a + b)\n");
    }

    #[test]
    fn test_print_expression_block() {
        let mut ast = create_test_ast();

        let block = Expression::Block(Block {
            entity: ast.create_entity(),
            label: None,
            statements: vec![
                Statement::Let(Let {
                    entity: ast.create_entity(),
                    pattern: Pattern::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "x".to_string(),
                    }),
                    type_expr: None,
                    value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                        entity: ast.create_entity(),
                        value: 5,
                    }))),
                    is_mutable: false,
                    is_static: false,
                }),
                Statement::Expression(Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "x".to_string(),
                })),
            ],
        });

        ast.root.statements.push(Statement::Expression(block));

        let output = print_ast(&ast);
        assert!(output.contains("{\n  let x = 5;\n  x\n}"));
    }

    #[test]
    fn test_print_if_with_multiple_branches() {
        let mut ast = create_test_ast();

        let if_expr = Expression::If(Box::new(If {
            entity: ast.create_entity(),
            branches: vec![
                IfBranch {
                    entity: ast.create_entity(),
                    condition: Expression::Operation(Operation {
                        entity: ast.create_entity(),
                        elements: vec![
                            OperationElement::Operand(Expression::Identifier(Identifier {
                                entity: ast.create_entity(),
                                name: "x".to_string(),
                            })),
                            OperationElement::Operator(">".to_string()),
                            OperationElement::Operand(Expression::Literal(Literal::Integer(
                                IntegerLiteral {
                                    entity: ast.create_entity(),
                                    value: 0,
                                },
                            ))),
                        ],
                    }),
                    body: Expression::Literal(Literal::String(StringLiteral {
                        entity: ast.create_entity(),
                        fragments: vec![StringFragment::Text("positive".to_string())],
                    })),
                },
                IfBranch {
                    entity: ast.create_entity(),
                    condition: Expression::Operation(Operation {
                        entity: ast.create_entity(),
                        elements: vec![
                            OperationElement::Operand(Expression::Identifier(Identifier {
                                entity: ast.create_entity(),
                                name: "x".to_string(),
                            })),
                            OperationElement::Operator("<".to_string()),
                            OperationElement::Operand(Expression::Literal(Literal::Integer(
                                IntegerLiteral {
                                    entity: ast.create_entity(),
                                    value: 0,
                                },
                            ))),
                        ],
                    }),
                    body: Expression::Literal(Literal::String(StringLiteral {
                        entity: ast.create_entity(),
                        fragments: vec![StringFragment::Text("negative".to_string())],
                    })),
                },
            ],
            else_branch: Some(Box::new(Expression::Literal(Literal::String(
                StringLiteral {
                    entity: ast.create_entity(),
                    fragments: vec![StringFragment::Text("zero".to_string())],
                },
            )))),
        }));

        ast.root.statements.push(Statement::Expression(if_expr));

        let output = print_ast(&ast);
        assert!(output.contains("if x > 0"));
        assert!(output.contains("\"positive\""));
        assert!(output.contains("else if x < 0"));
        assert!(output.contains("\"negative\""));
        assert!(output.contains("else"));
        assert!(output.contains("\"zero\""));
    }

    #[test]
    fn test_print_nested_if() {
        let mut ast = create_test_ast();

        let nested_if = Expression::If(Box::new(If {
            entity: ast.create_entity(),
            branches: vec![IfBranch {
                entity: ast.create_entity(),
                condition: Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "a".to_string(),
                }),
                body: Expression::Literal(Literal::Integer(IntegerLiteral {
                    entity: ast.create_entity(),
                    value: 1,
                })),
            }],
            else_branch: Some(Box::new(Expression::If(Box::new(If {
                entity: ast.create_entity(),
                branches: vec![IfBranch {
                    entity: ast.create_entity(),
                    condition: Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "b".to_string(),
                    }),
                    body: Expression::Literal(Literal::Integer(IntegerLiteral {
                        entity: ast.create_entity(),
                        value: 2,
                    })),
                }],
                else_branch: Some(Box::new(Expression::Literal(Literal::Integer(
                    IntegerLiteral {
                        entity: ast.create_entity(),
                        value: 3,
                    },
                )))),
            })))),
        }));

        ast.root.statements.push(Statement::Expression(nested_if));

        let output = print_ast(&ast);
        assert!(output.contains("if a"));
        assert!(output.contains("else if b"));
    }

    #[test]
    fn test_print_if_with_block_body() {
        let mut ast = create_test_ast();

        let if_expr = Expression::If(Box::new(If {
            entity: ast.create_entity(),
            branches: vec![IfBranch {
                entity: ast.create_entity(),
                condition: Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "cond".to_string(),
                }),
                body: Expression::Block(Block {
                    entity: ast.create_entity(),
                    label: None,
                    statements: vec![Statement::Expression(Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "x".to_string(),
                    }))],
                }),
            }],
            else_branch: None,
        }));

        ast.root.statements.push(Statement::Expression(if_expr));

        let output = print_ast(&ast);
        assert!(output.contains("if cond {\n  x\n}"));
    }
}
