# Semantic Analysis and AST Phase Separation Design for Patina

## Overview

This document outlines the design for implementing semantic analysis with phase-separated ASTs in the Patina programming language. Inspired by MacoCaml's phase separation techniques, this design creates a modular, type-safe compilation pipeline where each stage has its own specialized AST representation while maintaining code reuse through appropriate abstractions.

## Goals

### AST Phase Separation Goals

1. **Clear Phase Boundaries**: Distinct AST representations for each compilation phase
2. **Type-Safe Transitions**: Compile-time guarantees for valid phase transformations
3. **Optimized Representations**: Each phase's AST is tailored to its specific needs
4. **Progressive Refinement**: ASTs become more specialized as compilation progresses

### Semantic Analysis Goals

1. **Modular Design**: Each analysis phase should be independent and composable
2. **Code Reuse**: Share common infrastructure across different analyses
3. **Incremental Analysis**: Support analyzing partial programs and incremental updates
4. **Error Recovery**: Continue analysis after encountering errors
5. **Extensibility**: Easy to add new analysis passes

## Background: Phase Separation Motivation

Traditional single-AST designs suffer from:
- Mixing concerns across compilation phases
- Carrying unnecessary information through the pipeline
- Lack of type safety for phase-specific operations
- Difficulty in optimizing phase-specific traversals

MacoCaml's approach demonstrates the benefits of:
- **Phase Separation**: Clear distinction between compile-time and runtime code
- **Typed Source Calculus**: Type safety across phases
- **Quotation-based Staging**: Code can be quoted and spliced across phases
- **Compile-time Evaluation**: Macros execute during compilation

## Architecture

### Phase-Separated Compilation Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Phase-Separated AST Pipeline                      │
├─────────────────────────────────────────────────────────────────────┤
│  Source → Parse AST → Desugared AST → Typed AST → IR Tree → Output  │
│             ↓             ↓              ↓            ↓              │
│         Parsing      Name Res.      Type Check    Optimize          │
│                      + Desugar      + Inference   + Lower           │
└─────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────┐
│                       Shared Infrastructure                          │
├─────────────────────────────────────────────────────────────────────┤
│  • Symbol Tables    • Type Registry    • Error Collector            │
│  • Scope Manager    • Type Unifier     • Diagnostics               │
│  • Phase Traits     • Transform Traits • Analysis Context           │
└─────────────────────────────────────────────────────────────────────┘
```

### AST Phase Design

The compilation pipeline uses a hybrid approach combining phase-based AST hierarchy with trait-based separation for maximum type safety and code reuse:

#### Phase 1: Parse AST
- Raw syntactic representation from parser
- Preserves all source information (comments, formatting)
- No semantic information

```rust
pub mod parse_ast {
    use crate::Position;
    
    pub struct Module {
        pub position: Position,
        pub name: String,
        pub items: Vec<Item>,
    }
    
    pub enum Item {
        Statement(Statement),
        Declaration(Declaration),
    }
    
    pub struct Statement {
        pub position: Position,
        pub kind: StatementKind,
    }
}
```

#### Phase 2: Desugared AST
- Syntactic sugar expanded (e.g., `if` expressions to match)
- Macros expanded
- Names still unresolved

```rust
pub mod desugared_ast {
    use super::parse_ast;
    
    pub struct Module {
        pub parsed: parse_ast::Module,
        pub expanded_items: Vec<Item>,
    }
    
    // Simplified representations without sugar
    pub enum Expression {
        Match(MatchExpr),  // All conditionals become match
        Block(BlockExpr),
        // No If, While, etc. - all desugared
    }
}
```

#### Phase 3: Typed AST
- All names resolved to symbols
- Types inferred and checked
- Lifetime annotations added

```rust
pub mod typed_ast {
    use super::desugared_ast;
    use crate::analysis::{SymbolId, TypeId};
    
    pub struct Module {
        pub desugared: desugared_ast::Module,
        pub exports: HashMap<String, SymbolId>,
        pub type_env: TypeEnvironment,
    }
    
    pub struct Expression {
        pub desugared: desugared_ast::Expression,
        pub ty: TypeId,
        pub symbol_ref: Option<SymbolId>,
    }
}
```

#### Phase 4: IR Tree
- Low-level representation
- Control flow explicit
- Ready for optimization and code generation

```rust
pub mod ir_tree {
    pub struct Module {
        pub functions: Vec<Function>,
        pub globals: Vec<Global>,
    }
    
    pub struct Function {
        pub blocks: Vec<BasicBlock>,
        pub entry: BlockId,
    }
    
    pub struct BasicBlock {
        pub instructions: Vec<Instruction>,
        pub terminator: Terminator,
    }
}
```

### Phase Transformation Infrastructure

```rust
// Trait for phase transformations
pub trait Transform<From, To> {
    type Error;
    type Context;
    
    fn transform(from: From, ctx: &mut Self::Context) -> Result<To, Self::Error>;
}

// Example: Parse to Desugared transformation
pub struct Desugarer {
    macro_expander: MacroExpander,
}

impl Transform<parse_ast::Module, desugared_ast::Module> for Desugarer {
    type Error = DesugarError;
    type Context = AnalysisContext;
    
    fn transform(from: parse_ast::Module, ctx: &mut Self::Context) -> Result<desugared_ast::Module, Self::Error> {
        // Expand macros, desugar syntax
    }
}
```

### Phase-Aware Traits

```rust
// Traits that track compilation phase
pub trait CompilationPhase {
    type SymbolInfo;
    type TypeInfo;
    type Position;
}

pub struct ParsePhase;
impl CompilationPhase for ParsePhase {
    type SymbolInfo = ();  // No symbols yet
    type TypeInfo = ();    // No types yet
    type Position = Position;
}

pub struct TypedPhase;
impl CompilationPhase for TypedPhase {
    type SymbolInfo = SymbolId;
    type TypeInfo = TypeId;
    type Position = Position;
}

// Phase-aware visitor
pub trait PhaseVisitor<P: CompilationPhase> {
    fn visit_with_phase(&mut self, node: &dyn PhaseNode<P>);
}
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

## Design Evaluation

### Evaluation Criteria

1. **Type Safety**: How well does the approach prevent phase-related errors?
2. **Modularity**: How easy is it to add new phases or modify existing ones?
3. **Performance**: What is the runtime and compile-time overhead?
4. **Usability**: How complex is the API for compiler developers?
5. **Extensibility**: How well does it support future language features?

### Design Decision: Hybrid Approach

After evaluating multiple design options (phase-based hierarchy, parameterized ASTs with phantom types, trait-based separation, staging with quotation, and modular components), the recommended approach combines:

1. **Phase-based AST hierarchy** for clarity and type safety
2. **Trait-based separation** for code reuse and flexibility
3. **Shared infrastructure** for common functionality

This hybrid provides:
- Clear phase separation with distinct AST types
- Trait-based common functionality
- Type-safe transformations
- Extensible architecture
- Optimal balance of type safety, modularity, and usability

### Implementation Strategy

The implementation combines both AST phase separation and semantic analysis into a unified pipeline:

#### Phase 1: Foundation and Phase Infrastructure (Week 1-2)
1. Define core phase traits and transformation framework
2. Implement `SymbolTable` and `Scope` management
3. Create `ErrorCollector` and diagnostic infrastructure
4. Set up basic module structure for phase-specific ASTs

#### Phase 2: Parse AST and Initial Analysis (Week 3-4)
1. Implement parse_ast module with current AST structures
2. Add ParsePhase trait implementations
3. Build basic `NameResolver` visitor
4. Update parser to generate parse_ast nodes

#### Phase 3: Type System and Typed AST (Week 5-6)
1. Implement `TypeRegistry` and core type representations
2. Design typed_ast module with type annotations
3. Create `TypeChecker` visitor as Transform trait
4. Add type inference for simple cases

#### Phase 4: IR Generation and Advanced Analysis (Week 7-8)
1. Design IR tree structure
2. Implement lowering from typed_ast to IR
3. Add control flow analysis
4. Basic ownership/lifetime checking for pointer types

#### Phase 5: Integration and Optimization (Week 9-10)
1. Create analysis pipeline orchestrator
2. Update visitor pattern for phase-specific traversals
3. Add incremental analysis support
4. Performance testing and optimization

#### Phase 6: Documentation and Testing (Week 11-12)
1. Comprehensive documentation for phase system
2. Migration guide for existing code
3. Extensive test coverage for all phases
4. Integration tests for complete pipeline

## Usage Example

```rust
use patina::analysis::{analyze_pipeline, PipelineOptions};
use patina::ast::{parse_ast, desugared_ast, typed_ast};

// Parse source into phase 1 AST
let source = std::fs::read_to_string("example.pat")?;
let parse_tree: parse_ast::Module = parse_to_ast(&source)?;

// Option 1: Run complete pipeline
let options = PipelineOptions {
    type_check: true,
    check_ownership: true,
    optimize: true,
};

let ir = analyze_pipeline(parse_tree, options)?;

// Option 2: Run phases individually for more control
let mut ctx = AnalysisContext::new();

// Phase 1 → Phase 2: Desugar
let desugared = Desugarer::new()
    .transform(parse_tree, &mut ctx)?;

// Phase 2 → Phase 3: Name resolution and type checking
let typed = TypeChecker::new()
    .transform(desugared, &mut ctx)?;

// Access phase-specific information
let symbols = ctx.symbol_table();
let types = ctx.type_registry();
let diagnostics = ctx.diagnostics();

// Query typed AST
if let Some(main_fn) = typed.find_function("main") {
    println!("main function type: {:?}", main_fn.signature());
}

// Phase 3 → Phase 4: Generate IR
let ir = IRGenerator::new()
    .transform(typed, &mut ctx)?;
```

## Benefits of This Design

### Phase Separation Benefits

1. **Clear Phase Boundaries**: Each compilation stage has distinct responsibilities
2. **Type-Safe Transitions**: Invalid phase transformations caught at compile time
3. **Optimized Representations**: Each AST contains only phase-relevant information
4. **Progressive Refinement**: Information accumulates through phases without cluttering early stages
5. **Parallel Development**: Teams can work on different phases independently

### Semantic Analysis Benefits

1. **Modularity**: Each analysis phase is independent and composable
2. **Reusability**: Shared infrastructure reduces duplication across phases
3. **Extensibility**: New analyses can be added as visitors or transformers
4. **Performance**: Supports incremental analysis and caching
5. **Error Recovery**: Can continue analysis after encountering errors
6. **Type Safety**: Leverages Rust's type system for correctness

### Combined Architecture Benefits

1. **Clean Separation of Concerns**: Parsing, analysis, and code generation are clearly separated
2. **Flexible Pipeline**: Can run phases individually or as a complete pipeline
3. **Better Error Messages**: Phase-specific context enables precise diagnostics
4. **Easier Testing**: Each phase can be tested in isolation
5. **Future-Proof**: Architecture supports advanced features like macros and staging

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

1. **IDE Support**: Language server protocol implementation with phase-aware features
2. **Optimization Passes**: Dead code elimination, constant folding at IR level
3. **Code Generation**: LLVM or custom bytecode backend from IR tree
4. **Incremental Compilation**: Fine-grained dependency tracking per phase
5. **Parallel Analysis**: Concurrent analysis of independent modules
6. **Macro System**: Compile-time code generation with phase separation
7. **Staging Support**: Quotation-based metaprogramming inspired by MacoCaml

## Conclusion

This unified design for semantic analysis and AST phase separation provides a robust foundation for the Patina compiler. By combining:

1. **Phase-separated ASTs** that evolve through compilation stages
2. **Modular semantic analysis** with reusable infrastructure
3. **Type-safe transformations** between phases
4. **Visitor-based traversals** for extensibility

We achieve a compiler architecture that is:
- **Maintainable**: Clear separation of concerns and phase boundaries
- **Extensible**: Easy to add new phases, analyses, and optimizations
- **Correct**: Type system prevents phase-related errors
- **Performant**: Phase-specific representations and incremental analysis
- **User-friendly**: Better error messages and debugging support

The design balances theoretical soundness with practical implementation concerns, providing a clear path forward for building a modern systems programming language compiler.