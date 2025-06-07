//! Tests for the visitor pattern implementation

#[cfg(test)]
mod visitor_tests {
    use crate::ast::*;

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

    #[test]
    fn test_walk_empty_ast() {
        let ast = Ast::new();
        let mut visitor = CollectingVisitor {
            identifiers: vec![],
            literals: vec![],
        };

        walk_ast(&mut visitor, &ast);

        assert!(visitor.identifiers.is_empty());
        assert!(visitor.literals.is_empty());
    }

    #[test]
    fn test_walk_simple_let() {
        let mut ast = Ast::new();

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

        let mut visitor = CollectingVisitor {
            identifiers: vec![],
            literals: vec![],
        };

        walk_ast(&mut visitor, &ast);

        assert_eq!(visitor.identifiers, vec!["x"]);
        assert_eq!(visitor.literals, vec!["42"]);
    }

    #[test]
    fn test_walk_nested_expressions() {
        let mut ast = Ast::new();

        let entities: Vec<_> = (0..5).map(|_| ast.create_entity()).collect();

        let expr = Expression::Operation(Operation {
            entity: entities[0],
            elements: vec![
                OperationElement::Operand(Expression::Identifier(Identifier {
                    entity: entities[1],
                    name: "a".to_string(),
                })),
                OperationElement::Operator("+".to_string()),
                OperationElement::Operand(Expression::Identifier(Identifier {
                    entity: entities[2],
                    name: "b".to_string(),
                })),
            ],
        });

        ast.root.statements.push(Statement::Expression(expr));

        let mut visitor = CollectingVisitor {
            identifiers: vec![],
            literals: vec![],
        };

        walk_ast(&mut visitor, &ast);

        assert_eq!(visitor.identifiers.len(), 2);
        assert!(visitor.identifiers.contains(&"a".to_string()));
        assert!(visitor.identifiers.contains(&"b".to_string()));
    }

    #[test]
    fn test_walk_if_expression() {
        let mut ast = Ast::new();

        let entities: Vec<_> = (0..5).map(|_| ast.create_entity()).collect();

        let if_expr = Expression::If(Box::new(If {
            entity: entities[0],
            branches: vec![IfBranch {
                entity: entities[1],
                condition: Expression::Identifier(Identifier {
                    entity: entities[2],
                    name: "cond".to_string(),
                }),
                body: Expression::Literal(Literal::Integer(IntegerLiteral {
                    entity: entities[3],
                    value: 1,
                })),
            }],
            else_branch: Some(Box::new(Expression::Literal(Literal::Integer(
                IntegerLiteral {
                    entity: entities[4],
                    value: 2,
                },
            )))),
        }));

        ast.root.statements.push(Statement::Expression(if_expr));

        let mut visitor = CollectingVisitor {
            identifiers: vec![],
            literals: vec![],
        };

        walk_ast(&mut visitor, &ast);

        assert_eq!(visitor.identifiers, vec!["cond"]);
        assert_eq!(visitor.literals.len(), 2);
        assert!(visitor.literals.contains(&"1".to_string()));
        assert!(visitor.literals.contains(&"2".to_string()));
    }

    #[test]
    fn test_walk_function() {
        let mut ast = Ast::new();

        let entities: Vec<_> = (0..6).map(|_| ast.create_entity()).collect();

        let func = Statement::Function(Function {
            entity: entities[0],
            name: "test".to_string(),
            parameters: vec![Parameter {
                entity: entities[1],
                name: "x".to_string(),
                type_expr: Expression::Identifier(Identifier {
                    entity: entities[2],
                    name: "i32".to_string(),
                }),
            }],
            return_type: Some(Expression::Identifier(Identifier {
                entity: entities[3],
                name: "i32".to_string(),
            })),
            body: Some(Block {
                entity: entities[4],
                label: None,
                statements: vec![Statement::Expression(Expression::Identifier(Identifier {
                    entity: entities[5],
                    name: "x".to_string(),
                }))],
            }),
            is_static: false,
        });

        ast.root.statements.push(func);

        let mut visitor = CollectingVisitor {
            identifiers: vec![],
            literals: vec![],
        };

        walk_ast(&mut visitor, &ast);

        // Should have visited: i32 (param type), i32 (return type), x (body)
        assert_eq!(visitor.identifiers.len(), 3);
        assert_eq!(
            visitor.identifiers.iter().filter(|&s| s == "i32").count(),
            2
        );
        assert_eq!(visitor.identifiers.iter().filter(|&s| s == "x").count(), 1);
    }

    #[test]
    fn test_component_filter_visitor() {
        let mut ast = Ast::new();

        let entity1 = ast.create_entity();
        let entity2 = ast.create_entity();

        // Add Position component only to entity1
        ast.add_component(entity1, Position::default());

        let stmt1 = Statement::Expression(Expression::Identifier(Identifier {
            entity: entity1,
            name: "with_pos".to_string(),
        }));

        let stmt2 = Statement::Expression(Expression::Identifier(Identifier {
            entity: entity2,
            name: "no_pos".to_string(),
        }));

        ast.root.statements.push(stmt1);
        ast.root.statements.push(stmt2);

        // Create a visitor that only visits nodes with Position component
        let counter = CountingVisitor { count: 0 };
        let filter = |ast: &Ast, entity: Entity| ast.get_component::<Position>(entity).is_some();
        let mut filtered = ComponentFilterWrapper::new(counter, filter);

        walk_ast(&mut filtered, &ast);

        // Should only visit the identifier with Position component
        assert_eq!(filtered.inner.count, 1);
    }

    struct CountingVisitor {
        count: usize,
    }

    impl Visitor for CountingVisitor {
        fn visit_identifier(&mut self, _ast: &Ast, _ident: &Identifier) {
            self.count += 1;
        }
    }

    #[test]
    fn test_filter_traverser() {
        let mut ast = Ast::new();

        let entity1 = ast.create_entity();
        let entity2 = ast.create_entity();
        let entity3 = ast.create_entity();

        ast.add_component(entity1, Stage { stage_number: 1 });
        ast.add_component(entity2, Stage { stage_number: 2 });
        ast.add_component(entity3, Position::default());

        let traverser = FilterTraverser::<Stage>::new();
        let mut stages = Vec::new();

        traverser.traverse(&ast, |_entity, stage| {
            stages.push(stage.stage_number);
        });

        assert_eq!(stages.len(), 2);
        assert!(stages.contains(&1));
        assert!(stages.contains(&2));
    }
}
