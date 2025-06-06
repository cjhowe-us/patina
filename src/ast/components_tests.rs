//! Tests for AST components

#[cfg(test)]
mod tests {
    use crate::ast::components::*;

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos.filename, None);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);
        assert_eq!(pos.length, 0);
    }

    #[test]
    fn test_position_creation() {
        let pos = Position {
            filename: Some("test.pat".to_string()),
            line: 10,
            column: 5,
            offset: 100,
            length: 20,
        };
        assert_eq!(pos.filename, Some("test.pat".to_string()));
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.offset, 100);
        assert_eq!(pos.length, 20);
    }

    #[test]
    fn test_stage_component() {
        let stage = Stage { stage_number: 0 };
        assert_eq!(stage.stage_number, 0);

        let stage2 = Stage { stage_number: 5 };
        assert_eq!(stage2.stage_number, 5);
    }

    #[test]
    fn test_type_info_component() {
        let type_info = TypeInfo {
            type_name: "i32".to_string(),
            is_resolved: true,
        };
        assert_eq!(type_info.type_name, "i32");
        assert!(type_info.is_resolved);

        let unresolved = TypeInfo {
            type_name: "T".to_string(),
            is_resolved: false,
        };
        assert!(!unresolved.is_resolved);
    }

    #[test]
    fn test_symbol_info_component() {
        let symbol = SymbolInfo {
            name: "my_var".to_string(),
            scope_id: 42,
            is_defined: true,
        };
        assert_eq!(symbol.name, "my_var");
        assert_eq!(symbol.scope_id, 42);
        assert!(symbol.is_defined);
    }

    #[test]
    fn test_comment_component() {
        let comment = Comment {
            text: "This is a comment".to_string(),
            is_doc_comment: false,
        };
        assert_eq!(comment.text, "This is a comment");
        assert!(!comment.is_doc_comment);

        let doc_comment = Comment {
            text: "Documentation comment".to_string(),
            is_doc_comment: true,
        };
        assert!(doc_comment.is_doc_comment);
    }

    #[test]
    fn test_annotation_component() {
        let annotation = Annotation {
            name: "deprecated".to_string(),
            arguments: vec!["Use new_function instead".to_string()],
        };
        assert_eq!(annotation.name, "deprecated");
        assert_eq!(annotation.arguments.len(), 1);
        assert_eq!(annotation.arguments[0], "Use new_function instead");
    }

    #[test]
    fn test_error_component() {
        let error = Error {
            message: "Undefined variable".to_string(),
            error_type: ErrorType::NameError,
        };
        assert_eq!(error.message, "Undefined variable");
        assert_eq!(error.error_type, ErrorType::NameError);
    }

    #[test]
    fn test_error_types() {
        let parse_err = ErrorType::ParseError;
        let type_err = ErrorType::TypeError;
        let name_err = ErrorType::NameError;
        let stage_err = ErrorType::StageError;

        assert!(matches!(parse_err, ErrorType::ParseError));
        assert!(matches!(type_err, ErrorType::TypeError));
        assert!(matches!(name_err, ErrorType::NameError));
        assert!(matches!(stage_err, ErrorType::StageError));
    }

    #[test]
    fn test_component_box() {
        let pos = Position::default();
        let boxed = ComponentBox::new(pos.clone());

        let retrieved = boxed.downcast::<Position>();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().line, 1);

        let wrong_type = boxed.downcast::<Stage>();
        assert!(wrong_type.is_none());
    }

    #[test]
    fn test_component_box_mut() {
        let stage = Stage { stage_number: 1 };
        let mut boxed = ComponentBox::new(stage);

        {
            let mut_ref = boxed.downcast_mut::<Stage>();
            assert!(mut_ref.is_some());
            mut_ref.unwrap().stage_number = 2;
        }

        let retrieved = boxed.downcast::<Stage>();
        assert_eq!(retrieved.unwrap().stage_number, 2);
    }
}
