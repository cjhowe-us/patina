//! Additional tests for visitor pattern to improve coverage

#[cfg(test)]
mod tests {
    use crate::ast::*;

    struct CountingVisitor {
        visits: Vec<String>,
    }

    impl CountingVisitor {
        fn new() -> Self {
            Self { visits: Vec::new() }
        }
    }

    struct CollectingVisitor {
        identifiers: Vec<String>,
        literals: Vec<String>,
    }

    impl Visitor for CollectingVisitor {
        fn visit_identifier(&mut self, _ast: &Ast, ident: &Identifier) {
            self.identifiers.push(ident.name.clone());
        }

        fn visit_integer_literal(&mut self, _ast: &Ast, lit: &IntegerLiteral) {
            self.literals.push(lit.value.to_string());
        }

        fn visit_string_literal(&mut self, _ast: &Ast, lit: &StringLiteral) {
            let text = lit
                .fragments
                .iter()
                .filter_map(|f| match f {
                    StringFragment::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");
            if !text.is_empty() {
                self.literals.push(text);
            }
        }
    }

    impl Visitor for CountingVisitor {
        fn visit_module(&mut self, _ast: &Ast, module: &Module) {
            self.visits.push(format!("module:{}", module.name));
        }

        fn visit_integer_literal(&mut self, _ast: &Ast, lit: &IntegerLiteral) {
            self.visits.push(format!("integer:{}", lit.value));
        }

        fn visit_identifier(&mut self, _ast: &Ast, ident: &Identifier) {
            self.visits.push(format!("identifier:{}", ident.name));
        }

        fn visit_function(&mut self, _ast: &Ast, func: &Function) {
            self.visits.push(format!("function:{}", func.name));
        }

        fn visit_let(&mut self, _ast: &Ast, let_stmt: &Let) {
            self.visits.push(format!(
                "let:is_static={},is_mutable={}",
                let_stmt.is_static, let_stmt.is_mutable
            ));
        }

        fn visit_return(&mut self, _ast: &Ast, ret: &Return) {
            self.visits.push(format!("return:label={:?}", ret.label));
        }

        fn visit_while(&mut self, _ast: &Ast, while_stmt: &While) {
            self.visits
                .push(format!("while:label={:?}", while_stmt.label));
        }

        fn visit_for(&mut self, _ast: &Ast, for_stmt: &For) {
            self.visits.push(format!("for:label={:?}", for_stmt.label));
        }

        fn visit_break(&mut self, _ast: &Ast, break_stmt: &Break) {
            self.visits
                .push(format!("break:label={:?}", break_stmt.label));
        }

        fn visit_block(&mut self, _ast: &Ast, block: &Block) {
            self.visits.push(format!("block:label={:?}", block.label));
        }

        fn visit_static(&mut self, _ast: &Ast, _static_stmt: &Static) {
            self.visits.push("static".to_string());
        }

        fn visit_case(&mut self, _ast: &Ast, _case: &Case) {
            self.visits.push("case".to_string());
        }

        fn visit_operation(&mut self, _ast: &Ast, op: &Operation) {
            self.visits
                .push(format!("operation:elements={}", op.elements.len()));
        }

        fn visit_call(&mut self, _ast: &Ast, _call: &Call) {
            self.visits.push("call".to_string());
        }

        fn visit_index(&mut self, _ast: &Ast, _index: &Index) {
            self.visits.push("index".to_string());
        }

        fn visit_field(&mut self, _ast: &Ast, field: &Field) {
            self.visits.push(format!("field:{}", field.field));
        }

        fn visit_float_literal(&mut self, _ast: &Ast, lit: &FloatLiteral) {
            self.visits.push(format!("float:{}", lit.value));
        }

        fn visit_boolean_literal(&mut self, _ast: &Ast, lit: &BooleanLiteral) {
            self.visits.push(format!("bool:{}", lit.value));
        }

        fn visit_null_literal(&mut self, _ast: &Ast, _lit: &NullLiteral) {
            self.visits.push("null".to_string());
        }

        fn visit_tuple_pattern(&mut self, _ast: &Ast, pattern: &TuplePattern) {
            self.visits
                .push(format!("tuple_pattern:len={}", pattern.patterns.len()));
        }

        fn visit_struct_pattern(&mut self, _ast: &Ast, pattern: &StructPattern) {
            self.visits.push(format!("struct_pattern:{}", pattern.name));
        }

        fn visit_case_branch(&mut self, _ast: &Ast, branch: &CaseBranch) {
            self.visits
                .push(format!("case_branch:has_guard={}", branch.guard.is_some()));
        }
    }

    #[test]
    fn test_walk_module() {
        let mut ast = Ast::new();

        let func_entity = ast.create_entity();
        let module = Statement::Module(Module {
            entity: ast.create_entity(),
            name: "test_mod".to_string(),
            statements: vec![Statement::Function(Function {
                entity: func_entity,
                name: "inner_func".to_string(),
                parameters: vec![],
                return_type: None,
                body: None,
                is_static: false,
            })],
        });

        ast.root.statements.push(module);

        let mut visitor = CountingVisitor::new();
        walk_ast(&mut visitor, &ast);

        assert!(visitor.visits.contains(&"module:test_mod".to_string()));
        assert!(visitor.visits.contains(&"function:inner_func".to_string()));
    }

    #[test]
    fn test_walk_return() {
        let mut ast = Ast::new();

        let ret = Statement::Return(Return {
            entity: ast.create_entity(),
            label: Some("loop1".to_string()),
            value: Some(Expression::Literal(Literal::Integer(IntegerLiteral {
                entity: ast.create_entity(),
                value: 42,
            }))),
        });

        ast.root.statements.push(ret);

        let mut visitor = CountingVisitor::new();
        walk_ast(&mut visitor, &ast);

        assert!(visitor
            .visits
            .contains(&"return:label=Some(\"loop1\")".to_string()));
        assert!(visitor.visits.contains(&"integer:42".to_string()));
    }

    #[test]
    fn test_walk_while() {
        let mut ast = Ast::new();

        let while_stmt = Statement::While(While {
            entity: ast.create_entity(),
            label: Some("outer".to_string()),
            condition: Expression::Literal(Literal::Boolean(BooleanLiteral {
                entity: ast.create_entity(),
                value: true,
            })),
            body: Block {
                entity: ast.create_entity(),
                label: None,
                statements: vec![Statement::Break(Break {
                    entity: ast.create_entity(),
                    label: Some("outer".to_string()),
                    value: None,
                })],
            },
        });

        ast.root.statements.push(while_stmt);

        let mut visitor = CountingVisitor::new();
        walk_ast(&mut visitor, &ast);

        assert!(visitor
            .visits
            .contains(&"while:label=Some(\"outer\")".to_string()));
        assert!(visitor.visits.contains(&"bool:true".to_string()));
        assert!(visitor.visits.contains(&"block:label=None".to_string()));
        assert!(visitor
            .visits
            .contains(&"break:label=Some(\"outer\")".to_string()));
    }

    #[test]
    fn test_walk_for() {
        let mut ast = Ast::new();

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

        let mut visitor = CountingVisitor::new();
        walk_ast(&mut visitor, &ast);

        assert!(visitor.visits.contains(&"for:label=None".to_string()));
        assert!(visitor
            .visits
            .contains(&"let:is_static=false,is_mutable=true".to_string()));
    }

    #[test]
    fn test_walk_call_and_index() {
        let mut ast = Ast::new();

        let expr = Expression::Call(Call {
            entity: ast.create_entity(),
            function: Box::new(Expression::Field(Field {
                entity: ast.create_entity(),
                object: Box::new(Expression::Index(Index {
                    entity: ast.create_entity(),
                    object: Box::new(Expression::Identifier(Identifier {
                        entity: ast.create_entity(),
                        name: "array".to_string(),
                    })),
                    index: Box::new(Expression::Literal(Literal::Integer(IntegerLiteral {
                        entity: ast.create_entity(),
                        value: 0,
                    }))),
                })),
                field: "method".to_string(),
            })),
            arguments: vec![
                Expression::Literal(Literal::Float(FloatLiteral {
                    entity: ast.create_entity(),
                    value: std::f64::consts::PI,
                })),
                Expression::Literal(Literal::Null(NullLiteral {
                    entity: ast.create_entity(),
                })),
            ],
        });

        ast.root.statements.push(Statement::Expression(expr));

        let mut visitor = CountingVisitor::new();
        walk_ast(&mut visitor, &ast);

        assert!(visitor.visits.contains(&"call".to_string()));
        assert!(visitor.visits.contains(&"field:method".to_string()));
        assert!(visitor.visits.contains(&"index".to_string()));
        assert!(visitor
            .visits
            .contains(&format!("float:{}", std::f64::consts::PI)));
        assert!(visitor.visits.contains(&"null".to_string()));
    }

    #[test]
    fn test_walk_patterns() {
        let mut ast = Ast::new();

        let let_stmt = Statement::Let(Let {
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
                        Pattern::Tuple(TuplePattern {
                            entity: ast.create_entity(),
                            patterns: vec![
                                Pattern::Identifier(Identifier {
                                    entity: ast.create_entity(),
                                    name: "py1".to_string(),
                                }),
                                Pattern::Identifier(Identifier {
                                    entity: ast.create_entity(),
                                    name: "py2".to_string(),
                                }),
                            ],
                        }),
                    ),
                ],
            }),
            type_expr: None,
            value: None,
            is_mutable: false,
            is_static: true,
        });

        ast.root.statements.push(let_stmt);

        let mut visitor = CountingVisitor::new();
        walk_ast(&mut visitor, &ast);

        assert!(visitor
            .visits
            .contains(&"let:is_static=true,is_mutable=false".to_string()));
        assert!(visitor.visits.contains(&"struct_pattern:Point".to_string()));
        assert!(visitor.visits.contains(&"tuple_pattern:len=2".to_string()));
    }

    #[test]
    fn test_walk_case_with_guards() {
        let mut ast = Ast::new();

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

        let mut visitor = CountingVisitor::new();
        walk_ast(&mut visitor, &ast);

        assert!(visitor.visits.contains(&"case".to_string()));
        assert!(visitor
            .visits
            .contains(&"case_branch:has_guard=true".to_string()));
        assert!(visitor
            .visits
            .contains(&"case_branch:has_guard=false".to_string()));
    }

    #[test]
    fn test_walk_nested_blocks() {
        let mut ast = Ast::new();

        let block = Statement::Block(Block {
            entity: ast.create_entity(),
            label: Some("outer".to_string()),
            statements: vec![Statement::Block(Block {
                entity: ast.create_entity(),
                label: Some("inner".to_string()),
                statements: vec![Statement::Expression(Expression::Block(Block {
                    entity: ast.create_entity(),
                    label: None,
                    statements: vec![],
                }))],
            })],
        });

        ast.root.statements.push(block);

        let mut visitor = CountingVisitor::new();
        walk_ast(&mut visitor, &ast);

        assert_eq!(
            visitor
                .visits
                .iter()
                .filter(|v| v.starts_with("block:"))
                .count(),
            3
        );
        assert!(visitor
            .visits
            .contains(&"block:label=Some(\"outer\")".to_string()));
        assert!(visitor
            .visits
            .contains(&"block:label=Some(\"inner\")".to_string()));
        assert!(visitor.visits.contains(&"block:label=None".to_string()));
    }

    #[test]
    fn test_component_filter_with_multiple_nodes() {
        let mut ast = Ast::new();

        // Create multiple nodes with different components
        let entities: Vec<_> = (0..5).map(|_| ast.create_entity()).collect();

        // Add Stage component to some entities
        ast.add_component(entities[0], Stage { stage_number: 0 });
        ast.add_component(entities[2], Stage { stage_number: 1 });
        ast.add_component(entities[4], Stage { stage_number: 2 });

        // Add TypeInfo component to some entities
        ast.add_component(
            entities[1],
            TypeInfo {
                type_name: "i32".to_string(),
                is_resolved: true,
            },
        );
        ast.add_component(
            entities[2],
            TypeInfo {
                type_name: "String".to_string(),
                is_resolved: true,
            },
        );

        // Create AST with these entities
        ast.root.statements = vec![
            Statement::Expression(Expression::Identifier(Identifier {
                entity: entities[0],
                name: "a".to_string(),
            })),
            Statement::Expression(Expression::Identifier(Identifier {
                entity: entities[1],
                name: "b".to_string(),
            })),
            Statement::Expression(Expression::Identifier(Identifier {
                entity: entities[2],
                name: "c".to_string(),
            })),
            Statement::Expression(Expression::Identifier(Identifier {
                entity: entities[3],
                name: "d".to_string(),
            })),
            Statement::Expression(Expression::Identifier(Identifier {
                entity: entities[4],
                name: "e".to_string(),
            })),
        ];

        // Test filter for Stage component
        let stage_visitor = CollectingVisitor {
            identifiers: vec![],
            literals: vec![],
        };
        let filter = |ast: &Ast, entity: Entity| ast.get_component::<Stage>(entity).is_some();
        let mut stage_filter = ComponentFilterWrapper::new(stage_visitor, filter);
        walk_ast(&mut stage_filter, &ast);

        assert_eq!(stage_filter.inner.identifiers.len(), 3);
        assert!(stage_filter.inner.identifiers.contains(&"a".to_string()));
        assert!(stage_filter.inner.identifiers.contains(&"c".to_string()));
        assert!(stage_filter.inner.identifiers.contains(&"e".to_string()));

        // Test filter for TypeInfo component
        let type_visitor = CollectingVisitor {
            identifiers: vec![],
            literals: vec![],
        };
        let filter = |ast: &Ast, entity: Entity| ast.get_component::<TypeInfo>(entity).is_some();
        let mut type_filter = ComponentFilterWrapper::new(type_visitor, filter);
        walk_ast(&mut type_filter, &ast);

        assert_eq!(type_filter.inner.identifiers.len(), 2);
        assert!(type_filter.inner.identifiers.contains(&"b".to_string()));
        assert!(type_filter.inner.identifiers.contains(&"c".to_string()));
    }
}
