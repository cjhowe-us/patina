use crate::analysis::{ErrorCollector, Stage, SymbolTable, TypeRegistry};
use std::sync::{Arc, Mutex};

/// Analysis context that holds shared state for semantic analysis
#[derive(Clone)]
pub struct AnalysisContext {
    symbol_table: Arc<Mutex<SymbolTable>>,
    type_registry: Arc<Mutex<TypeRegistry>>,
    diagnostics: Arc<Mutex<ErrorCollector>>,
    current_stage: Arc<Mutex<Stage>>,
}

impl AnalysisContext {
    pub fn new() -> Self {
        Self::with_stage(Stage::RUNTIME)
    }
    
    pub fn with_stage(stage: Stage) -> Self {
        Self {
            symbol_table: Arc::new(Mutex::new(SymbolTable::new_with_stage(stage))),
            type_registry: Arc::new(Mutex::new(TypeRegistry::new())),
            diagnostics: Arc::new(Mutex::new(ErrorCollector::new())),
            current_stage: Arc::new(Mutex::new(stage)),
        }
    }
    
    /// Access the symbol table
    pub fn with_symbol_table<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut SymbolTable) -> R,
    {
        let mut table = self.symbol_table.lock().unwrap();
        f(&mut table)
    }
    
    /// Access the type registry
    pub fn with_type_registry<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut TypeRegistry) -> R,
    {
        let mut registry = self.type_registry.lock().unwrap();
        f(&mut registry)
    }
    
    /// Access the diagnostics
    pub fn with_diagnostics<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ErrorCollector) -> R,
    {
        let mut diagnostics = self.diagnostics.lock().unwrap();
        f(&mut diagnostics)
    }
    
    /// Get the current stage
    pub fn current_stage(&self) -> Stage {
        *self.current_stage.lock().unwrap()
    }
    
    /// Set the current stage
    pub fn set_stage(&self, stage: Stage) {
        *self.current_stage.lock().unwrap() = stage;
        
        // Also update the symbol table's stage
        self.with_symbol_table(|table| {
            table.set_stage(stage);
        });
    }
    
    /// Enter a new stage for a scoped operation
    pub fn with_stage_scope<F, R>(&self, stage: Stage, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let prev_stage = self.current_stage();
        self.set_stage(stage);
        let result = f();
        self.set_stage(prev_stage);
        result
    }
    
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.with_diagnostics(|diag| diag.has_errors())
    }
    
    /// Get a snapshot of all diagnostics
    pub fn take_diagnostics(&self) -> Vec<crate::analysis::Diagnostic> {
        self.with_diagnostics(|diag| {
            let diagnostics = diag.diagnostics().to_vec();
            diag.clear();
            diagnostics
        })
    }
    
    /// Create a fork of this context with independent diagnostics
    pub fn fork(&self) -> Self {
        Self {
            symbol_table: Arc::clone(&self.symbol_table),
            type_registry: Arc::clone(&self.type_registry),
            diagnostics: Arc::new(Mutex::new(ErrorCollector::new())),
            current_stage: Arc::clone(&self.current_stage),
        }
    }
    
    /// Merge diagnostics from another context
    pub fn merge_diagnostics(&self, other: &Self) {
        let other_diagnostics = other.take_diagnostics();
        self.with_diagnostics(|diag| {
            for diagnostic in other_diagnostics {
                diag.add_diagnostic(diagnostic);
            }
        });
    }
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::Type;
    use crate::ast::Position;
    
    #[test]
    fn test_context_creation() {
        let ctx = AnalysisContext::new();
        assert_eq!(ctx.current_stage(), Stage::RUNTIME);
        assert!(!ctx.has_errors());
        
        let ctx2 = AnalysisContext::with_stage(Stage::COMPILE_TIME);
        assert_eq!(ctx2.current_stage(), Stage::COMPILE_TIME);
    }
    
    #[test]
    fn test_symbol_table_access() {
        let ctx = AnalysisContext::new();
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };
        
        let symbol_id = ctx.with_symbol_table(|table| {
            table.declare_variable("x".to_string(), pos, false, false).unwrap()
        });
        
        let found = ctx.with_symbol_table(|table| {
            table.lookup("x")
        });
        
        assert_eq!(found, Some(symbol_id));
    }
    
    #[test]
    fn test_type_registry_access() {
        let ctx = AnalysisContext::new();
        
        let i32_type = ctx.with_type_registry(|reg| {
            reg.get_builtin("i32").unwrap()
        });
        
        let array_type = ctx.with_type_registry(|reg| {
            reg.make_array_type(i32_type, Some(10))
        });
        
        ctx.with_type_registry(|reg| {
            let ty = reg.get_type(array_type).unwrap();
            assert!(matches!(ty, Type::Array { .. }));
        });
    }
    
    #[test]
    fn test_diagnostics_access() {
        let ctx = AnalysisContext::new();
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };
        
        ctx.with_diagnostics(|diag| {
            diag.error("Test error".to_string(), pos.clone());
            diag.warning("Test warning".to_string(), pos);
        });
        
        assert!(ctx.has_errors());
        
        let diagnostics = ctx.take_diagnostics();
        assert_eq!(diagnostics.len(), 2);
        assert!(!ctx.has_errors()); // Should be cleared
    }
    
    #[test]
    fn test_stage_management() {
        let ctx = AnalysisContext::new();
        assert_eq!(ctx.current_stage(), Stage::RUNTIME);
        
        ctx.set_stage(Stage::COMPILE_TIME);
        assert_eq!(ctx.current_stage(), Stage::COMPILE_TIME);
        
        // Check that symbol table stage is also updated
        ctx.with_symbol_table(|table| {
            assert_eq!(table.current_stage(), Stage::COMPILE_TIME);
        });
    }
    
    #[test]
    fn test_stage_scope() {
        let ctx = AnalysisContext::new();
        assert_eq!(ctx.current_stage(), Stage::RUNTIME);
        
        let result = ctx.with_stage_scope(Stage::COMPILE_TIME, || {
            assert_eq!(ctx.current_stage(), Stage::COMPILE_TIME);
            42
        });
        
        assert_eq!(result, 42);
        assert_eq!(ctx.current_stage(), Stage::RUNTIME); // Should be restored
    }
    
    #[test]
    fn test_context_fork() {
        let ctx = AnalysisContext::new();
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };
        
        // Add symbol to original context
        ctx.with_symbol_table(|table| {
            table.declare_variable("x".to_string(), pos.clone(), false, false).unwrap();
        });
        
        // Fork the context
        let forked = ctx.fork();
        
        // Symbol should be visible in forked context
        let found = forked.with_symbol_table(|table| {
            table.lookup("x").is_some()
        });
        assert!(found);
        
        // Add error to forked context
        forked.with_diagnostics(|diag| {
            diag.error("Forked error".to_string(), pos);
        });
        
        // Original context should not have the error
        assert!(!ctx.has_errors());
        assert!(forked.has_errors());
    }
    
    #[test]
    fn test_merge_diagnostics() {
        let ctx1 = AnalysisContext::new();
        let ctx2 = AnalysisContext::new();
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };
        
        ctx1.with_diagnostics(|diag| {
            diag.error("Error 1".to_string(), pos.clone());
        });
        
        ctx2.with_diagnostics(|diag| {
            diag.error("Error 2".to_string(), pos);
        });
        
        ctx1.merge_diagnostics(&ctx2);
        
        let diagnostics = ctx1.take_diagnostics();
        assert_eq!(diagnostics.len(), 2);
    }
}