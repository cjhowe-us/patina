use crate::analysis::{Arena, Id, Stage, TypeId};
use crate::ast::Position;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Arena<Scope>,
    symbols: Arena<Symbol>,
    current_scope: Id<Scope>,
    current_stage: Stage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub parent: Option<Id<Scope>>,
    pub symbols: HashMap<String, Id<Symbol>>,
    pub kind: ScopeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeKind {
    Global,
    Module { name: String },
    Function { name: String },
    Block { label: Option<String> },
    Lambda,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub position: Position,
    pub ty: Option<TypeId>,
    pub scope: Id<Scope>,
    pub stage: Stage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable {
        mutable: bool,
        static_: bool,
    },
    Function {
        params: Vec<ParamInfo>,
        return_type: Option<TypeId>,
    },
    Type {
        definition: TypeDefinition,
    },
    Module,
    Label,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamInfo {
    pub name: String,
    pub ty: Option<TypeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefinition {
    Struct,
    Enum,
    Alias(TypeId),
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::new_with_stage(Stage::RUNTIME)
    }

    pub fn new_with_stage(stage: Stage) -> Self {
        let mut scopes = Arena::new();
        let global_scope = Scope {
            parent: None,
            symbols: HashMap::new(),
            kind: ScopeKind::Global,
        };
        let current_scope = scopes.insert(global_scope);

        Self {
            scopes,
            symbols: Arena::new(),
            current_scope,
            current_stage: stage,
        }
    }

    pub fn declare_variable(
        &mut self,
        name: String,
        position: Position,
        mutable: bool,
        static_: bool,
    ) -> Result<Id<Symbol>, SymbolError> {
        self.declare_symbol(name, SymbolKind::Variable { mutable, static_ }, position)
    }

    pub fn declare_function(
        &mut self,
        name: String,
        position: Position,
        params: Vec<ParamInfo>,
        return_type: Option<TypeId>,
    ) -> Result<Id<Symbol>, SymbolError> {
        self.declare_symbol(
            name,
            SymbolKind::Function {
                params,
                return_type,
            },
            position,
        )
    }

    pub fn declare_label(
        &mut self,
        name: String,
        position: Position,
    ) -> Result<Id<Symbol>, SymbolError> {
        self.declare_symbol(name, SymbolKind::Label, position)
    }

    fn declare_symbol(
        &mut self,
        name: String,
        kind: SymbolKind,
        position: Position,
    ) -> Result<Id<Symbol>, SymbolError> {
        let current_scope = self
            .scopes
            .get_mut(self.current_scope)
            .ok_or(SymbolError::InvalidScope)?;

        if current_scope.symbols.contains_key(&name) {
            return Err(SymbolError::DuplicateSymbol { name, position });
        }

        let symbol = Symbol {
            name: name.clone(),
            kind,
            position,
            ty: None,
            scope: self.current_scope,
            stage: self.current_stage,
        };

        let symbol_id = self.symbols.insert(symbol);
        current_scope.symbols.insert(name, symbol_id);

        Ok(symbol_id)
    }

    pub fn lookup(&self, name: &str) -> Option<Id<Symbol>> {
        self.lookup_in_scope(name, self.current_scope)
    }

    fn lookup_in_scope(&self, name: &str, scope_id: Id<Scope>) -> Option<Id<Symbol>> {
        let scope = self.scopes.get(scope_id)?;

        if let Some(symbol_id) = scope.symbols.get(name) {
            return Some(*symbol_id);
        }

        if let Some(parent) = scope.parent {
            self.lookup_in_scope(name, parent)
        } else {
            None
        }
    }

    pub fn enter_scope(&mut self, kind: ScopeKind) -> Id<Scope> {
        let new_scope = Scope {
            parent: Some(self.current_scope),
            symbols: HashMap::new(),
            kind,
        };

        let scope_id = self.scopes.insert(new_scope);
        self.current_scope = scope_id;
        scope_id
    }

    pub fn exit_scope(&mut self) -> Result<(), SymbolError> {
        let current_scope = self
            .scopes
            .get(self.current_scope)
            .ok_or(SymbolError::InvalidScope)?;

        if let Some(parent) = current_scope.parent {
            self.current_scope = parent;
            Ok(())
        } else {
            Err(SymbolError::ExitGlobalScope)
        }
    }

    pub fn current_scope(&self) -> Id<Scope> {
        self.current_scope
    }

    pub fn get_symbol(&self, id: Id<Symbol>) -> Option<&Symbol> {
        self.symbols.get(id)
    }

    pub fn get_symbol_mut(&mut self, id: Id<Symbol>) -> Option<&mut Symbol> {
        self.symbols.get_mut(id)
    }

    pub fn get_scope(&self, id: Id<Scope>) -> Option<&Scope> {
        self.scopes.get(id)
    }

    pub fn is_global_scope(&self) -> bool {
        matches!(
            self.scopes.get(self.current_scope).map(|s| &s.kind),
            Some(ScopeKind::Global)
        )
    }

    // Stage-aware methods

    pub fn current_stage(&self) -> Stage {
        self.current_stage
    }

    pub fn set_stage(&mut self, stage: Stage) {
        self.current_stage = stage;
    }

    pub fn declare_staged(
        &mut self,
        name: String,
        position: Position,
        stage: Stage,
        kind: SymbolKind,
    ) -> Result<Id<Symbol>, SymbolError> {
        let prev_stage = self.current_stage;
        self.current_stage = stage;
        let result = self.declare_symbol(name, kind, position);
        self.current_stage = prev_stage;
        result
    }

    pub fn lookup_at_stage(&self, name: &str, current_stage: Stage) -> Option<Id<Symbol>> {
        // Find symbol and check if it's visible at the given stage
        self.lookup(name).filter(|&id| {
            if let Some(symbol) = self.get_symbol(id) {
                current_stage.can_see_symbol_at(symbol.stage)
            } else {
                false
            }
        })
    }

    pub fn lookup_at_current_stage(&self, name: &str) -> Option<Id<Symbol>> {
        self.lookup_at_stage(name, self.current_stage)
    }

    pub fn lookup_all_visible_stages(
        &self,
        name: &str,
        current_stage: Stage,
    ) -> Vec<(Id<Symbol>, Stage)> {
        let mut results = Vec::new();
        let mut scope_id = self.current_scope;

        loop {
            if let Some(scope) = self.scopes.get(scope_id) {
                if let Some(&symbol_id) = scope.symbols.get(name) {
                    if let Some(symbol) = self.get_symbol(symbol_id) {
                        if current_stage.can_see_symbol_at(symbol.stage) {
                            results.push((symbol_id, symbol.stage));
                        }
                    }
                }

                if let Some(parent) = scope.parent {
                    scope_id = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        results
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolError {
    DuplicateSymbol { name: String, position: Position },
    InvalidScope,
    ExitGlobalScope,
}

impl std::fmt::Display for SymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolError::DuplicateSymbol { name, position } => {
                write!(f, "Duplicate symbol '{name}' at {position:?}")
            }
            SymbolError::InvalidScope => write!(f, "Invalid scope"),
            SymbolError::ExitGlobalScope => write!(f, "Cannot exit global scope"),
        }
    }
}

impl std::error::Error for SymbolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_creation() {
        let table = SymbolTable::new();
        assert!(table.is_global_scope());
    }

    #[test]
    fn test_variable_declaration() {
        let mut table = SymbolTable::new();
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };

        let result = table.declare_variable("x".to_string(), pos, false, false);
        assert!(result.is_ok());

        let symbol_id = result.unwrap();
        let symbol = table.get_symbol(symbol_id).unwrap();
        assert_eq!(symbol.name, "x");
        assert_eq!(
            symbol.kind,
            SymbolKind::Variable {
                mutable: false,
                static_: false
            }
        );
    }

    #[test]
    fn test_duplicate_declaration() {
        let mut table = SymbolTable::new();
        let pos1 = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };
        let pos2 = Position {
            filename: None,
            line: 2,
            column: 1,
            offset: 0,
            length: 0,
        };

        let result1 = table.declare_variable("x".to_string(), pos1, false, false);
        assert!(result1.is_ok());

        let result2 = table.declare_variable("x".to_string(), pos2, false, false);
        assert!(matches!(result2, Err(SymbolError::DuplicateSymbol { .. })));
    }

    #[test]
    fn test_symbol_lookup() {
        let mut table = SymbolTable::new();
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };

        let symbol_id = table
            .declare_variable("x".to_string(), pos, false, false)
            .unwrap();

        let lookup_result = table.lookup("x");
        assert_eq!(lookup_result, Some(symbol_id));

        let missing_lookup = table.lookup("y");
        assert_eq!(missing_lookup, None);
    }

    #[test]
    fn test_scope_management() {
        let mut table = SymbolTable::new();
        let pos1 = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };
        let pos2 = Position {
            filename: None,
            line: 2,
            column: 1,
            offset: 0,
            length: 0,
        };

        // Declare in global scope
        let global_var = table
            .declare_variable("global".to_string(), pos1, false, false)
            .unwrap();

        // Enter function scope
        let _func_scope = table.enter_scope(ScopeKind::Function {
            name: "test".to_string(),
        });

        // Declare in function scope
        let local_var = table
            .declare_variable("local".to_string(), pos2, false, false)
            .unwrap();

        // Both should be visible
        assert_eq!(table.lookup("global"), Some(global_var));
        assert_eq!(table.lookup("local"), Some(local_var));

        // Exit function scope
        table.exit_scope().unwrap();

        // Only global should be visible
        assert_eq!(table.lookup("global"), Some(global_var));
        assert_eq!(table.lookup("local"), None);
    }

    #[test]
    fn test_shadowing_in_nested_scope() {
        let mut table = SymbolTable::new();
        let pos1 = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };
        let pos2 = Position {
            filename: None,
            line: 2,
            column: 1,
            offset: 0,
            length: 0,
        };

        // Declare in global scope
        let global_x = table
            .declare_variable("x".to_string(), pos1, false, false)
            .unwrap();

        // Enter block scope
        table.enter_scope(ScopeKind::Block { label: None });

        // Shadow in block scope
        let local_x = table
            .declare_variable("x".to_string(), pos2, false, false)
            .unwrap();

        // Should find local x
        assert_eq!(table.lookup("x"), Some(local_x));
        assert_ne!(global_x, local_x);

        // Exit block scope
        table.exit_scope().unwrap();

        // Should find global x again
        assert_eq!(table.lookup("x"), Some(global_x));
    }

    #[test]
    fn test_function_declaration() {
        let mut table = SymbolTable::new();
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };

        let params = vec![
            ParamInfo {
                name: "a".to_string(),
                ty: None,
            },
            ParamInfo {
                name: "b".to_string(),
                ty: None,
            },
        ];

        let func_id = table
            .declare_function("add".to_string(), pos, params.clone(), None)
            .unwrap();

        let symbol = table.get_symbol(func_id).unwrap();
        assert_eq!(symbol.name, "add");
        assert_eq!(
            symbol.kind,
            SymbolKind::Function {
                params,
                return_type: None
            }
        );
    }

    #[test]
    fn test_exit_global_scope_error() {
        let mut table = SymbolTable::new();
        let result = table.exit_scope();
        assert!(matches!(result, Err(SymbolError::ExitGlobalScope)));
    }

    // Stage-aware tests

    #[test]
    fn test_symbol_table_with_stage() {
        let table = SymbolTable::new_with_stage(Stage::COMPILE_TIME);
        assert_eq!(table.current_stage(), Stage::COMPILE_TIME);

        let table2 = SymbolTable::new();
        assert_eq!(table2.current_stage(), Stage::RUNTIME);
    }

    #[test]
    fn test_declare_staged() {
        let mut table = SymbolTable::new_with_stage(Stage::RUNTIME);
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };

        // Declare a compile-time constant
        let const_id = table
            .declare_staged(
                "BUFFER_SIZE".to_string(),
                pos.clone(),
                Stage::COMPILE_TIME,
                SymbolKind::Variable {
                    mutable: false,
                    static_: true,
                },
            )
            .unwrap();

        let symbol = table.get_symbol(const_id).unwrap();
        assert_eq!(symbol.stage, Stage::COMPILE_TIME);
        assert_eq!(table.current_stage(), Stage::RUNTIME); // Should restore original stage
    }

    #[test]
    fn test_stage_visibility() {
        let mut table = SymbolTable::new_with_stage(Stage::COMPILE_TIME);
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };

        // Declare symbols at different stages
        let stage0_sym = table
            .declare_variable("compile_val".to_string(), pos.clone(), false, false)
            .unwrap();

        table.set_stage(Stage::RUNTIME);
        let stage1_sym = table
            .declare_variable("runtime_val".to_string(), pos.clone(), false, false)
            .unwrap();

        table.set_stage(Stage::new(2));
        let stage2_sym = table
            .declare_variable("generated_val".to_string(), pos, false, false)
            .unwrap();

        // Test visibility at different stages

        // At stage 0, can only see stage 0 symbols
        assert_eq!(
            table.lookup_at_stage("compile_val", Stage::COMPILE_TIME),
            Some(stage0_sym)
        );
        assert_eq!(
            table.lookup_at_stage("runtime_val", Stage::COMPILE_TIME),
            None
        );
        assert_eq!(
            table.lookup_at_stage("generated_val", Stage::COMPILE_TIME),
            None
        );

        // At stage 1, can see stage 0 and 1 symbols
        assert_eq!(
            table.lookup_at_stage("compile_val", Stage::RUNTIME),
            Some(stage0_sym)
        );
        assert_eq!(
            table.lookup_at_stage("runtime_val", Stage::RUNTIME),
            Some(stage1_sym)
        );
        assert_eq!(table.lookup_at_stage("generated_val", Stage::RUNTIME), None);

        // At stage 2, can see all symbols
        let stage2 = Stage::new(2);
        assert_eq!(
            table.lookup_at_stage("compile_val", stage2),
            Some(stage0_sym)
        );
        assert_eq!(
            table.lookup_at_stage("runtime_val", stage2),
            Some(stage1_sym)
        );
        assert_eq!(
            table.lookup_at_stage("generated_val", stage2),
            Some(stage2_sym)
        );
    }

    #[test]
    fn test_lookup_all_visible_stages() {
        let mut table = SymbolTable::new_with_stage(Stage::COMPILE_TIME);
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };

        // Declare same name at different stages
        let x_stage0 = table
            .declare_variable("x".to_string(), pos.clone(), false, false)
            .unwrap();

        table.enter_scope(ScopeKind::Block { label: None });
        table.set_stage(Stage::RUNTIME);
        let x_stage1 = table
            .declare_variable("x".to_string(), pos, false, false)
            .unwrap();

        // At stage 0, only see stage 0 symbol
        let visible_at_0 = table.lookup_all_visible_stages("x", Stage::COMPILE_TIME);
        assert_eq!(visible_at_0.len(), 1);
        assert_eq!(visible_at_0[0], (x_stage0, Stage::COMPILE_TIME));

        // At stage 1, see both symbols
        let visible_at_1 = table.lookup_all_visible_stages("x", Stage::RUNTIME);
        assert_eq!(visible_at_1.len(), 2);
        assert!(visible_at_1.contains(&(x_stage0, Stage::COMPILE_TIME)));
        assert!(visible_at_1.contains(&(x_stage1, Stage::RUNTIME)));
    }

    #[test]
    fn test_lookup_at_current_stage() {
        let mut table = SymbolTable::new_with_stage(Stage::new(2));
        let pos = Position {
            filename: None,
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        };

        // Add symbol at stage 0
        table.set_stage(Stage::COMPILE_TIME);
        let early_sym = table
            .declare_variable("early".to_string(), pos.clone(), false, false)
            .unwrap();

        // Add symbol at stage 3
        table.set_stage(Stage::new(3));
        let _late_sym = table
            .declare_variable("late".to_string(), pos, false, false)
            .unwrap();

        // Set current stage to 2
        table.set_stage(Stage::new(2));

        // Can see early symbol (stage 0)
        assert_eq!(table.lookup_at_current_stage("early"), Some(early_sym));

        // Cannot see late symbol (stage 3)
        assert_eq!(table.lookup_at_current_stage("late"), None);
    }
}
