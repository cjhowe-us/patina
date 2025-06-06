//! Tests for the AST printer

#[cfg(test)]
mod tests {
    use crate::ast::*;

    fn print_ast(ast: &Ast) -> String {
        let mut printer = PrinterVisitor::new();
        printer.print_root(&ast.root);
        printer.output
    }

    fn create_test_ast() -> Ast {
        let mut ast = Ast::new();

        // Helper to create entities
        let mut entities = Vec::new();
        for _ in 0..20 {
            entities.push(ast.create_entity());
        }

        ast
    }

    #[test]
    fn test_print_empty_ast() {
        let ast = Ast::new();
        let output = print_ast(&ast);
        assert_eq!(output, "");
    }

    #[test]
    fn test_print_simple_let() {
        let mut ast = create_test_ast();

        let id_entity = ast.create_entity();
        let lit_entity = ast.create_entity();
        let let_entity = ast.create_entity();

        let let_stmt = Statement::Let(Let {
            entity: let_entity,
            pattern: Pattern::Identifier(Identifier {
                entity: id_entity,
                name: "x".to_string(),
            }),
            type_expr: None,
            value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                entity: lit_entity,
                value: 42,
            }))),
            is_mutable: false,
            is_static: false,
        });

        ast.root.statements.push(let_stmt);

        let output = print_ast(&ast);
        assert_eq!(output, "let x = 42;\n");
    }

    #[test]
    fn test_print_mutable_let() {
        let mut ast = create_test_ast();

        let let_stmt = Statement::Let(Let {
            entity: ast.create_entity(),
            pattern: Pattern::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "x".to_string(),
            }),
            type_expr: None,
            value: None,
            is_mutable: true,
            is_static: false,
        });

        ast.root.statements.push(let_stmt);

        let output = print_ast(&ast);
        assert_eq!(output, "let mut x;\n");
    }

    #[test]
    fn test_print_let_with_type() {
        let mut ast = create_test_ast();

        let let_stmt = Statement::Let(Let {
            entity: ast.create_entity(),
            pattern: Pattern::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "x".to_string(),
            }),
            type_expr: Some(Expression::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "i32".to_string(),
            })),
            value: None,
            is_mutable: false,
            is_static: false,
        });

        ast.root.statements.push(let_stmt);

        let output = print_ast(&ast);
        assert_eq!(output, "let x: i32;\n");
    }

    #[test]
    fn test_print_function() {
        let mut ast = create_test_ast();

        let func = Statement::Function(Function {
            entity: ast.create_entity(),
            name: "add".to_string(),
            parameters: vec![
                Parameter {
                    entity: ast.create_entity(),
                    name: "a".to_string(),
                    type_expr: Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "i32".to_string(),
                    }),
                },
                Parameter {
                    entity: ast.create_entity(),
                    name: "b".to_string(),
                    type_expr: Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "i32".to_string(),
                    }),
                },
            ],
            return_type: Some(Expression::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "i32".to_string(),
            })),
            body: Some(Block {
                entity: ast.create_entity(),
                label: None,
                statements: vec![Statement::Expression(Expression::Operation(Operation {
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
                }))],
            }),
            is_static: false,
        });

        ast.root.statements.push(func);

        let output = print_ast(&ast);
        assert_eq!(output, "fn add(a: i32, b: i32): i32 {\n  a + b\n}\n");
    }

    #[test]
    fn test_print_if_expression() {
        let mut ast = create_test_ast();

        let if_expr = Expression::If(Box::new(If {
            entity: ast.create_entity(),
            branches: vec![IfBranch {
                entity: ast.create_entity(),
                condition: Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "cond".to_string(),
                }),
                body: Expression::Literal(Literal::Integer(IntegerLiteral {
                    entity: ast.create_entity(),
                    value: 1,
                })),
            }],
            else_branch: Some(Box::new(Expression::Literal(Literal::Integer(
                IntegerLiteral {
                    entity: ast.create_entity(),
                    value: 2,
                },
            )))),
        }));

        ast.root.statements.push(Statement::Expression(if_expr));

        let output = print_ast(&ast);
        assert_eq!(output, "if cond {\n  1\n} else {\n  2\n}\n");
    }

    #[test]
    fn test_print_string_literal() {
        let mut ast = create_test_ast();

        let string_lit = Expression::Literal(Literal::String(StringLiteral {
            entity: ast.create_entity(),
            fragments: vec![
                StringFragment::Text("Hello, ".to_string()),
                StringFragment::Interpolation(Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "name".to_string(),
                })),
                StringFragment::Text("!".to_string()),
            ],
        }));

        ast.root.statements.push(Statement::Expression(string_lit));

        let output = print_ast(&ast);
        assert_eq!(output, "\"Hello, ${name}!\"\n");
    }

    #[test]
    fn test_print_operation() {
        let mut ast = create_test_ast();

        let op = Expression::Operation(Operation {
            entity: ast.create_entity(),
            elements: vec![
                OperationElement::Operand(Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "x".to_string(),
                })),
                OperationElement::Operator("*".to_string()),
                OperationElement::Operand(Expression::Literal(Literal::Integer(IntegerLiteral {
                    entity: ast.create_entity(),
                    value: 2,
                }))),
                OperationElement::Operator("+".to_string()),
                OperationElement::Operand(Expression::Literal(Literal::Integer(IntegerLiteral {
                    entity: ast.create_entity(),
                    value: 1,
                }))),
            ],
        });

        ast.root.statements.push(Statement::Expression(op));

        let output = print_ast(&ast);
        assert_eq!(output, "x * 2 + 1\n");
    }

    #[test]
    fn test_print_lambda() {
        let mut ast = create_test_ast();

        let lambda = Expression::Lambda(Box::new(Lambda {
            entity: ast.create_entity(),
            parameters: vec![Parameter {
                entity: ast.create_entity(),
                name: "x".to_string(),
                type_expr: Expression::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "i32".to_string(),
                }),
            }],
            return_type: Some(Box::new(Expression::Identifier(Identifier {
                entity: ast.create_entity(),
                name: "i32".to_string(),
            }))),
            body: Box::new(Expression::Operation(Operation {
                entity: ast.create_entity(),
                elements: vec![
                    OperationElement::Operand(Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "x".to_string(),
                    })),
                    OperationElement::Operator("*".to_string()),
                    OperationElement::Operand(Expression::Literal(Literal::Integer(
                        IntegerLiteral {
                            entity: ast.create_entity(),
                            value: 2,
                        },
                    ))),
                ],
            })),
        }));

        ast.root.statements.push(Statement::Expression(lambda));

        let output = print_ast(&ast);
        assert_eq!(output, "\\(x: i32): i32 => x * 2\n");
    }

    #[test]
    fn test_print_static_block() {
        let mut ast = create_test_ast();

        let static_stmt = Statement::Static(Static {
            entity: ast.create_entity(),
            statements: vec![Statement::Let(Let {
                entity: ast.create_entity(),
                pattern: Pattern::Identifier(Identifier {
                    entity: ast.create_entity(),
                    name: "x".to_string(),
                }),
                type_expr: None,
                value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                    entity: ast.create_entity(),
                    value: 42,
                }))),
                is_mutable: false,
                is_static: false,
            })],
        });

        ast.root.statements.push(static_stmt);

        let output = print_ast(&ast);
        assert_eq!(output, "static {\n  let x = 42;\n}\n");
    }

    #[test]
    fn test_print_module() {
        let mut ast = create_test_ast();

        let module = Statement::Module(Module {
            entity: ast.create_entity(),
            name: "math".to_string(),
            statements: vec![Statement::Function(Function {
                entity: ast.create_entity(),
                name: "square".to_string(),
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
                body: Some(Block {
                    entity: ast.create_entity(),
                    label: None,
                    statements: vec![Statement::Expression(Expression::Operation(Operation {
                        entity: ast.create_entity(),
                        elements: vec![
                            OperationElement::Operand(Expression::Identifier(Identifier {
                                entity: ast.create_entity(),
                                name: "x".to_string(),
                            })),
                            OperationElement::Operator("*".to_string()),
                            OperationElement::Operand(Expression::Identifier(Identifier {
                                entity: ast.create_entity(),
                                name: "x".to_string(),
                            })),
                        ],
                    }))],
                }),
                is_static: false,
            })],
        });

        ast.root.statements.push(module);

        let output = print_ast(&ast);
        assert_eq!(
            output,
            "module math {\n  fn square(x: i32): i32 {\n    x * x\n  }\n}\n"
        );
    }
}
