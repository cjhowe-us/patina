# Semantic Analysis Design for Patina

## Overview

This document outlines the design for implementing semantic analysis in the Patina programming language. The design emphasizes code reuse through appropriate abstractions while maintaining clean separation of concerns.

## Goals

1. **Modular Design**: Each analysis phase should be independent and composable
2. **Code Reuse**: Share common infrastructure across different analyses
3. **Incremental Analysis**: Support analyzing partial programs and incremental updates
4. **Error Recovery**: Continue analysis after encountering errors
5. **Extensibility**: Easy to add new analysis passes

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                         Analysis Pipeline                     │
├─────────────────────────────────────────────────────────────┤
│  AST → Name Resolution → Type Checking → Flow Analysis → IR  │
└─────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────┐
│                    Shared Infrastructure                      │
├─────────────────────────────────────────────────────────────┤
│  • Symbol Tables    • Type Registry    • Error Collector     │
│  • Scope Manager    • Type Unifier     • Diagnostics        │
└─────────────────────────────────────────────────────────────┘
```

### Phase 1: Symbol Collection and Name Resolution

#### 1.1 Symbol Table Design

```rust
// src/analysis/symbol_table.rs
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current_scope: ScopeId,
    symbols: HashMap<SymbolId, Symbol>,
}

pub struct Scope {
    id: ScopeId,
    parent: Option<ScopeId>,
    symbols: HashMap<String, SymbolId>,
    kind: ScopeKind,
}

pub enum ScopeKind {
    Global,
    Module { name: String },
    Function { name: String },
    Block { label: Option<String> },
    Lambda,
}

pub struct Symbol {
    id: SymbolId,
    name: String,
    kind: SymbolKind,
    position: Position,
    ty: Option<TypeId>,
    scope: ScopeId,
}

pub enum SymbolKind {
    Variable { mutable: bool, static_: bool },
    Function { params: Vec<ParamInfo>, return_type: Option<TypeId> },
    Type { definition: TypeDefinition },
    Module,
    Label,
}
```

#### 1.2 Name Resolution Visitor

```rust
// src/analysis/name_resolution.rs
pub struct NameResolver {
    symbol_table: SymbolTable,
    errors: ErrorCollector,
}

impl Visitor<(), NameResolutionResult> for NameResolver {
    // Collect declarations on enter, resolve references on exit
}
```

### Phase 2: Type System Infrastructure

#### 2.1 Type Registry

```rust
// src/analysis/types.rs
pub struct TypeRegistry {
    types: HashMap<TypeId, Type>,
    type_cache: HashMap<String, TypeId>, // For named types
}

pub enum Type {
    Integer { signed: bool, bits: u16 },
    Float { bits: u8 },
    Boolean,
    String,
    Array { element: TypeId, size: Option<usize> },
    Pointer { target: TypeId, modifier: PointerModifier },
    Function { params: Vec<TypeId>, return_type: Option<TypeId> },
    Struct { fields: Vec<Field> },
    Enum { variants: Vec<Variant> },
    Generic { name: String, constraints: Vec<Constraint> },
    Concrete { base: TypeId, args: Vec<TypeId> }, // Instantiated generic
    Unknown, // For type inference
    Error,   // For error recovery
}

pub enum PointerModifier {
    None,
    Mutable,
    Owned,
    Raw,
}
```

#### 2.2 Type Checking Visitor

```rust
// src/analysis/type_checker.rs
pub struct TypeChecker {
    symbol_table: Arc<SymbolTable>,
    type_registry: TypeRegistry,
    type_env: TypeEnvironment,
    errors: ErrorCollector,
}

impl Visitor<TypeContext, TypeResult> for TypeChecker {
    // Perform type checking and inference
}
```

### Phase 3: Advanced Analysis

#### 3.1 Control Flow Analysis

```rust
// src/analysis/control_flow.rs
pub struct ControlFlowAnalyzer {
    cfg: ControlFlowGraph,
    reachability: ReachabilityInfo,
}

pub struct ControlFlowGraph {
    blocks: Vec<BasicBlock>,
    edges: Vec<Edge>,
}
```

#### 3.2 Lifetime and Ownership Analysis

```rust
// src/analysis/ownership.rs
pub struct OwnershipAnalyzer {
    borrow_checker: BorrowChecker,
    lifetime_resolver: LifetimeResolver,
}
```

### Shared Infrastructure

#### Error Collection and Reporting

```rust
// src/analysis/diagnostics.rs
pub struct ErrorCollector {
    errors: Vec<Diagnostic>,
    warnings: Vec<Diagnostic>,
}

pub struct Diagnostic {
    level: DiagnosticLevel,
    message: String,
    position: Position,
    notes: Vec<Note>,
    suggestions: Vec<Suggestion>,
}

pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}
```

#### Analysis Context

```rust
// src/analysis/context.rs
pub struct AnalysisContext {
    symbol_table: Arc<SymbolTable>,
    type_registry: Arc<TypeRegistry>,
    diagnostics: Arc<Mutex<ErrorCollector>>,
    source_map: SourceMap,
}
```

### Visitor Framework Extensions

#### Enhanced Visitor Trait

```rust
// src/analysis/visitor_ext.rs
pub trait AnalysisVisitor<Input, Output> {
    type State;
    
    fn initial_state(&self) -> Self::State;
    
    fn enter_node(
        &mut self,
        node: &dyn SyntaxNode,
        input: &Input,
        state: &mut Self::State,
        ctx: &AnalysisContext,
    ) -> Result<(), AnalysisError>;
    
    fn exit_node(
        &mut self,
        node: &dyn SyntaxNode,
        input: &Input,
        state: &mut Self::State,
        ctx: &AnalysisContext,
    ) -> Result<Output, AnalysisError>;
}
```

### Implementation Strategy

#### Phase 1: Foundation (Week 1-2)
1. Implement `SymbolTable` and `Scope` management
2. Create `ErrorCollector` and diagnostic infrastructure
3. Build basic `NameResolver` visitor

#### Phase 2: Type System (Week 3-4)
1. Implement `TypeRegistry` and core type representations
2. Create `TypeChecker` visitor with basic type checking
3. Add type inference for simple cases

#### Phase 3: Advanced Features (Week 5-6)
1. Implement generic type handling
2. Add control flow analysis
3. Basic ownership/lifetime checking for pointer types

#### Phase 4: Integration (Week 7)
1. Create analysis pipeline orchestrator
2. Add incremental analysis support
3. Comprehensive error reporting

## Usage Example

```rust
use patina::analysis::{analyze, AnalysisOptions};

let ast = parse_file("example.pat")?;

let options = AnalysisOptions {
    type_check: true,
    check_ownership: true,
    warn_unused: true,
};

let result = analyze(&ast, options)?;

// Access analysis results
let symbols = result.symbol_table();
let types = result.type_registry();
let diagnostics = result.diagnostics();

// Query specific information
if let Some(symbol) = symbols.lookup("main") {
    let ty = types.get_type(symbol.ty);
    println!("main function type: {:?}", ty);
}
```

## Benefits of This Design

1. **Modularity**: Each analysis phase is independent
2. **Reusability**: Shared infrastructure reduces duplication
3. **Extensibility**: New analyses can be added as visitors
4. **Performance**: Supports incremental analysis
5. **Error Recovery**: Can continue after errors
6. **Type Safety**: Leverages Rust's type system

## Necessary Abstractions for Simple Implementation

### Core Abstractions

#### 1. `Id<T>` Pattern for Type Safety
```rust
// src/analysis/id.rs
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id<T> {
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(index: usize) -> Self {
        Self { index, _phantom: PhantomData }
    }
    
    pub fn index(self) -> usize {
        self.index
    }
}

// Type aliases for clarity
pub type SymbolId = Id<Symbol>;
pub type TypeId = Id<Type>;
pub type ScopeId = Id<Scope>;
```

**Benefits**: Prevents mixing up IDs, provides type safety at zero runtime cost.

#### 2. `Arena<T>` for Memory Management
```rust
// src/analysis/arena.rs
pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn insert(&mut self, item: T) -> Id<T> {
        let id = Id::new(self.items.len());
        self.items.push(item);
        id
    }
    
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        self.items.get(id.index())
    }
    
    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        self.items.get_mut(id.index())
    }
}
```

**Benefits**: Simple allocation strategy, stable references, efficient lookup.

#### 3. `VisitorDriver` for Standardized Traversal
```rust
// src/analysis/driver.rs
pub struct VisitorDriver<V> {
    visitor: V,
}

impl<V> VisitorDriver<V> {
    pub fn new(visitor: V) -> Self {
        Self { visitor }
    }
    
    pub fn visit_with_context<T>(&mut self, node: &dyn SyntaxNode, ctx: &AnalysisContext) -> T
    where
        V: AnalysisVisitor<AnalysisContext, T>,
    {
        // Standard pre/post-order traversal with error handling
    }
}
```

**Benefits**: Eliminates boilerplate, consistent error handling, standardized traversal.

### Keeping Implementation Simple

#### Strategy 1: Start with Minimal Viable Implementation

**Principle**: Implement only what's needed for basic functionality, then extend.

**Example**: Start with simple types (integers, booleans) before adding generics.

#### Strategy 2: Use Builder Pattern for Complex Construction
```rust
// src/analysis/builders.rs
pub struct SymbolBuilder {
    symbol: Symbol,
}

impl SymbolBuilder {
    pub fn new(name: String, position: Position) -> Self {
        Self {
            symbol: Symbol {
                name,
                position,
                kind: SymbolKind::Variable { mutable: false, static_: false },
                ty: None,
                scope: ScopeId::new(0),
            }
        }
    }
    
    pub fn mutable(mut self) -> Self {
        if let SymbolKind::Variable { ref mut mutable, .. } = self.symbol.kind {
            *mutable = true;
        }
        self
    }
    
    pub fn build(self) -> Symbol {
        self.symbol
    }
}
```

**Benefits**: Fluent API, reduces constructor complexity, optional parameters.

#### Strategy 3: Progressive Implementation Layers

**Layer 1: Data Structures Only**
```rust
// Just define the types, no logic yet
pub struct SymbolTable { /* fields */ }
pub struct TypeRegistry { /* fields */ }
```

**Layer 2: Basic Operations**
```rust
impl SymbolTable {
    pub fn new() -> Self { /* */ }
    pub fn insert(&mut self, symbol: Symbol) -> SymbolId { /* */ }
    pub fn lookup(&self, name: &str) -> Option<SymbolId> { /* */ }
}
```

**Layer 3: Advanced Features**
```rust
impl SymbolTable {
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId { /* */ }
    pub fn exit_scope(&mut self) { /* */ }
    pub fn resolve_path(&self, path: &[String]) -> Option<SymbolId> { /* */ }
}
```

## Step-by-Step Implementation Guide

### Phase 1: Foundation Setup

#### Step 1.1: Create Analysis Module Structure
```bash
mkdir src/analysis
touch src/analysis/mod.rs
touch src/analysis/id.rs
touch src/analysis/arena.rs
touch src/analysis/symbol_table.rs
touch src/analysis/diagnostics.rs
```

#### Step 1.2: Implement Core Abstractions
1. **Implement `Id<T>` in `src/analysis/id.rs`**:
   - Copy the code above
   - Add `Display` and `From<usize>` implementations
   - Export type aliases

2. **Implement `Arena<T>` in `src/analysis/arena.rs`**:
   - Copy the code above
   - Add iterator support: `impl<T> IntoIterator for Arena<T>`
   - Add `len()` and `is_empty()` methods

3. **Set up module exports in `src/analysis/mod.rs`**:
```rust
pub mod id;
pub mod arena;
pub mod symbol_table;
pub mod diagnostics;

pub use id::*;
pub use arena::*;
pub use symbol_table::*;
pub use diagnostics::*;
```

#### Step 1.3: Implement Symbol Table
1. **Define basic types in `src/analysis/symbol_table.rs`**:
   - Start with simple `Symbol` and `Scope` structs
   - Implement only `Global` and `Function` scope kinds initially
   - Use `Arena<Scope>` and `Arena<Symbol>` for storage

2. **Implement core operations**:
```rust
impl SymbolTable {
    pub fn new() -> Self {
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
        }
    }
    
    pub fn declare_variable(&mut self, name: String, position: Position) -> Result<SymbolId, String> {
        // Check for redeclaration in current scope
        // Insert symbol
        // Return SymbolId
    }
    
    pub fn lookup(&self, name: &str) -> Option<SymbolId> {
        // Search current scope, then parents
    }
}
```

#### Step 1.4: Implement Diagnostics
1. **Create `src/analysis/diagnostics.rs`**:
```rust
pub struct ErrorCollector {
    diagnostics: Vec<Diagnostic>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self { diagnostics: Vec::new() }
    }
    
    pub fn error(&mut self, message: String, position: Position) {
        self.diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            message,
            position,
            notes: Vec::new(),
        });
    }
    
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| matches!(d.level, DiagnosticLevel::Error))
    }
}
```

### Phase 2: Name Resolution Implementation

#### Step 2.1: Create Name Resolution Visitor
1. **Create `src/analysis/name_resolution.rs`**:
```rust
use crate::ast::{SyntaxNode, StatementKind, ExpressionKind};
use crate::visitor::Visitor;

pub struct NameResolver {
    symbol_table: SymbolTable,
    errors: ErrorCollector,
}

impl NameResolver {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: ErrorCollector::new(),
        }
    }
}
```

2. **Implement visitor for specific node types**:
```rust
impl Visitor<(), ()> for NameResolver {
    fn enter_statement(&mut self, stmt: &StatementKind, _: &()) -> Result<(), Error> {
        match stmt {
            StatementKind::Let { binding, .. } => {
                // Extract variable name from binding
                // Declare in symbol table
                // Handle errors
            }
            StatementKind::Function { name, .. } => {
                // Declare function symbol
                // Enter function scope
            }
            _ => {}
        }
        Ok(())
    }
    
    fn exit_statement(&mut self, stmt: &StatementKind, _: &()) -> Result<(), Error> {
        match stmt {
            StatementKind::Function { .. } => {
                // Exit function scope
            }
            _ => {}
        }
        Ok(())
    }
}
```

#### Step 2.2: Test Name Resolution
1. **Create test in `src/analysis/name_resolution.rs`**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_simple_variable_declaration() {
        let input = "let x = 42";
        let ast = parse(input).unwrap();
        
        let mut resolver = NameResolver::new();
        resolver.visit(&ast);
        
        assert!(!resolver.errors.has_errors());
        assert!(resolver.symbol_table.lookup("x").is_some());
    }
}
```

### Phase 3: Type System Foundation

#### Step 3.1: Implement Type Registry
1. **Create `src/analysis/types.rs`** with minimal type system:
```rust
pub struct TypeRegistry {
    types: Arena<Type>,
    builtins: HashMap<String, TypeId>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            types: Arena::new(),
            builtins: HashMap::new(),
        };
        
        // Add builtin types
        let i32_type = registry.types.insert(Type::Integer { signed: true, bits: 32 });
        registry.builtins.insert("i32".to_string(), i32_type);
        
        registry
    }
}
```

#### Step 3.2: Integrate with Existing Visitor
1. **Extend `NameResolver` to collect type information**:
```rust
impl Visitor<(), ()> for NameResolver {
    fn enter_expression(&mut self, expr: &ExpressionKind, _: &()) -> Result<(), Error> {
        match expr {
            ExpressionKind::Atom { atom } => {
                // Resolve variable references
                // Check if symbol exists
            }
            _ => {}
        }
        Ok(())
    }
}
```

### Testing Strategy

#### Unit Tests for Each Component
```rust
#[cfg(test)]
mod tests {
    #[test] fn test_symbol_table_basic_operations() { /* */ }
    #[test] fn test_scope_management() { /* */ }
    #[test] fn test_error_collection() { /* */ }
    #[test] fn test_type_registry() { /* */ }
}
```

#### Integration Tests
```rust
// tests/analysis_integration.rs
#[test]
fn test_complete_analysis_pipeline() {
    let source = r#"
        fun add(a: i32, b: i32): i32 {
            let result = a + b
            return result
        }
    "#;
    
    let ast = parse(source).unwrap();
    let analysis_result = analyze(&ast).unwrap();
    
    // Verify symbols were collected
    // Verify types were inferred
    // Verify no errors occurred
}
```

This approach ensures:
1. **Incremental complexity**: Start simple, add features gradually
2. **Testable components**: Each piece can be tested independently
3. **Clear abstractions**: `Id<T>` and `Arena<T>` eliminate complexity
4. **Reusable patterns**: Builder pattern and visitor driver reduce boilerplate

## Future Extensions

1. **IDE Support**: Language server protocol implementation
2. **Optimization Passes**: Dead code elimination, constant folding
3. **Code Generation**: LLVM or custom bytecode backend
4. **Incremental Compilation**: Fine-grained dependency tracking
5. **Parallel Analysis**: Concurrent analysis of independent modules