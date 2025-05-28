use crate::ast::Position;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ErrorCollector {
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub position: Position,
    pub notes: Vec<Note>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Note {
    pub message: String,
    pub position: Option<Position>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub replacement: Option<String>,
    pub position: Option<Position>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn error(&mut self, message: String, position: Position) {
        self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message,
            position,
            notes: Vec::new(),
            suggestions: Vec::new(),
        });
    }

    pub fn warning(&mut self, message: String, position: Position) {
        self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Warning,
            message,
            position,
            notes: Vec::new(),
            suggestions: Vec::new(),
        });
    }

    pub fn info(&mut self, message: String, position: Position) {
        self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Info,
            message,
            position,
            notes: Vec::new(),
            suggestions: Vec::new(),
        });
    }

    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.level, DiagnosticLevel::Error))
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.level, DiagnosticLevel::Error))
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.level, DiagnosticLevel::Warning))
            .count()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn take_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn merge(&mut self, other: ErrorCollector) {
        self.diagnostics.extend(other.diagnostics);
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Diagnostic {
    pub fn error(message: String, position: Position) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message,
            position,
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn warning(message: String, position: Position) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message,
            position,
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn with_note(mut self, message: String, position: Option<Position>) -> Self {
        self.notes.push(Note { message, position });
        self
    }

    pub fn with_suggestion(
        mut self,
        message: String,
        replacement: Option<String>,
        position: Option<Position>,
    ) -> Self {
        self.suggestions.push(Suggestion {
            message,
            replacement,
            position,
        });
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = match self.level {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Info => "info",
        };

        writeln!(
            f,
            "{}: {} at line {}, column {}",
            level_str, self.message, self.position.line, self.position.column
        )?;

        for note in &self.notes {
            write!(f, "  note: {}", note.message)?;
            if let Some(pos) = &note.position {
                write!(f, " at line {}, column {}", pos.line, pos.column)?;
            }
            writeln!(f)?;
        }

        for suggestion in &self.suggestions {
            write!(f, "  suggestion: {}", suggestion.message)?;
            if let Some(replacement) = &suggestion.replacement {
                write!(f, " (replace with: {replacement})")?;
            }
            if let Some(pos) = &suggestion.position {
                write!(f, " at line {}, column {}", pos.line, pos.column)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl fmt::Display for ErrorCollector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in &self.diagnostics {
            write!(f, "{diagnostic}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_collector_basic() {
        let mut collector = ErrorCollector::new();
        let pos = Position { 
            filename: None,
            line: 1, 
            column: 10,
            offset: 0,
        };

        collector.error("Undefined variable".to_string(), pos);
        
        assert!(collector.has_errors());
        assert_eq!(collector.error_count(), 1);
        assert_eq!(collector.warning_count(), 0);
        assert_eq!(collector.len(), 1);
    }

    #[test]
    fn test_mixed_diagnostics() {
        let mut collector = ErrorCollector::new();
        let pos1 = Position { 
            filename: None,
            line: 1, 
            column: 10,
            offset: 0,
        };
        let pos2 = Position { 
            filename: None,
            line: 2, 
            column: 5,
            offset: 0,
        };
        let pos3 = Position { 
            filename: None,
            line: 3, 
            column: 1,
            offset: 0,
        };

        collector.error("Error message".to_string(), pos1);
        collector.warning("Warning message".to_string(), pos2);
        collector.info("Info message".to_string(), pos3);

        assert!(collector.has_errors());
        assert_eq!(collector.error_count(), 1);
        assert_eq!(collector.warning_count(), 1);
        assert_eq!(collector.len(), 3);
    }

    #[test]
    fn test_diagnostic_builder() {
        let pos = Position { 
            filename: None,
            line: 10, 
            column: 15,
            offset: 0,
        };
        let note_pos = Position { 
            filename: None,
            line: 8, 
            column: 1,
            offset: 0,
        };

        let diagnostic = Diagnostic::error("Type mismatch".to_string(), pos)
            .with_note("Expected i32".to_string(), Some(note_pos))
            .with_suggestion(
                "Change type to i32".to_string(),
                Some("i32".to_string()),
                None,
            );

        assert_eq!(diagnostic.level, DiagnosticLevel::Error);
        assert_eq!(diagnostic.notes.len(), 1);
        assert_eq!(diagnostic.suggestions.len(), 1);
    }

    #[test]
    fn test_collector_merge() {
        let mut collector1 = ErrorCollector::new();
        let mut collector2 = ErrorCollector::new();
        let pos1 = Position { 
            filename: None,
            line: 1, 
            column: 1,
            offset: 0,
        };
        let pos2 = Position { 
            filename: None,
            line: 2, 
            column: 1,
            offset: 0,
        };

        collector1.error("Error 1".to_string(), pos1);
        collector2.warning("Warning 1".to_string(), pos2);

        collector1.merge(collector2);

        assert_eq!(collector1.len(), 2);
        assert_eq!(collector1.error_count(), 1);
        assert_eq!(collector1.warning_count(), 1);
    }

    #[test]
    fn test_diagnostic_display() {
        let pos = Position { 
            filename: None,
            line: 5, 
            column: 10,
            offset: 0,
        };
        let diagnostic = Diagnostic::error("Undefined variable 'x'".to_string(), pos)
            .with_note("Variable must be declared before use".to_string(), None);

        let output = format!("{}", diagnostic);
        assert!(output.contains("error: Undefined variable 'x' at line 5, column 10"));
        assert!(output.contains("note: Variable must be declared before use"));
    }

    #[test]
    fn test_clear_and_take() {
        let mut collector = ErrorCollector::new();
        let pos = Position { 
            filename: None,
            line: 1, 
            column: 1,
            offset: 0,
        };

        collector.error("Error".to_string(), pos.clone());
        assert_eq!(collector.len(), 1);

        collector.clear();
        assert_eq!(collector.len(), 0);
        assert!(collector.is_empty());

        collector.error("Another error".to_string(), pos);
        let diagnostics = collector.take_diagnostics();
        assert_eq!(diagnostics.len(), 1);
    }
}