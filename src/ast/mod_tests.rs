//! Tests for AST module

#[cfg(test)]
mod tests {
    use crate::ast::*;

    #[test]
    fn test_ast_creation() {
        let ast = Ast::new();
        assert_eq!(ast.root.statements.len(), 0);
        assert!(ast.component_entities.is_empty());
    }

    #[test]
    fn test_create_entity() {
        let mut ast = Ast::new();
        let entity1 = ast.create_entity();
        let entity2 = ast.create_entity();
        assert_ne!(entity1, entity2);
    }

    #[test]
    fn test_add_and_get_component() {
        let mut ast = Ast::new();
        let entity = ast.create_entity();
        
        let pos = Position {
            filename: Some("test.pat".to_string()),
            line: 10,
            column: 5,
            offset: 100,
            length: 20,
        };
        
        ast.add_component(entity, pos.clone());
        
        let retrieved = ast.get_component::<Position>(entity);
        assert!(retrieved.is_some());
        let retrieved_pos = retrieved.unwrap();
        assert_eq!(retrieved_pos.line, 10);
        assert_eq!(retrieved_pos.column, 5);
    }

    #[test]
    fn test_get_component_mut() {
        let mut ast = Ast::new();
        let entity = ast.create_entity();
        
        let stage = Stage { stage_number: 1 };
        ast.add_component(entity, stage);
        
        {
            let stage_ref = ast.get_component_mut::<Stage>(entity);
            assert!(stage_ref.is_some());
            stage_ref.unwrap().stage_number = 2;
        }
        
        let retrieved = ast.get_component::<Stage>(entity);
        assert_eq!(retrieved.unwrap().stage_number, 2);
    }

    #[test]
    fn test_missing_component() {
        let mut ast = Ast::new();
        let entity = ast.create_entity();
        
        let result = ast.get_component::<Position>(entity);
        assert!(result.is_none());
    }

    #[test]
    fn test_filter_entities() {
        let mut ast = Ast::new();
        let entity1 = ast.create_entity();
        let entity2 = ast.create_entity();
        let entity3 = ast.create_entity();
        
        ast.add_component(entity1, Position::default());
        ast.add_component(entity2, Position::default());
        ast.add_component(entity3, Stage { stage_number: 0 });
        
        let position_entities = ast.filter_entities::<Position>();
        assert_eq!(position_entities.len(), 2);
        assert!(position_entities.contains(&entity1));
        assert!(position_entities.contains(&entity2));
        assert!(!position_entities.contains(&entity3));
    }

    #[test]
    fn test_query_entities() {
        let mut ast = Ast::new();
        let entity = ast.create_entity();
        
        ast.add_component(entity, Position::default());
        ast.add_component(entity, Stage { stage_number: 1 });
        
        let mut count = 0;
        for (e, (pos, stage)) in ast.query_entities::<(&Position, &Stage)>().iter() {
            assert_eq!(e, entity);
            assert_eq!(pos.line, 1);
            assert_eq!(stage.stage_number, 1);
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn test_multiple_components() {
        let mut ast = Ast::new();
        let entity = ast.create_entity();
        
        ast.add_component(entity, Position::default());
        ast.add_component(entity, Stage { stage_number: 0 });
        ast.add_component(entity, SymbolInfo {
            name: "x".to_string(),
            scope_id: 1,
            is_defined: true,
        });
        
        assert!(ast.get_component::<Position>(entity).is_some());
        assert!(ast.get_component::<Stage>(entity).is_some());
        assert!(ast.get_component::<SymbolInfo>(entity).is_some());
    }
}