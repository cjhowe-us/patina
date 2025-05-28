# Semantic Analysis and Compile Stage Level Design for Patina

## Overview

This document outlines the design for implementing semantic analysis with compile stage levels in the Patina programming language. Inspired by MacroCaml's stage separation techniques, this design creates a modular, type-safe compilation pipeline where code can exist at different compile-time stages, enabling powerful metaprogramming capabilities while maintaining type safety.

## Goals

### Compile Stage Level Goals

1. **Clear Stage Boundaries**: Code exists at distinct compile-time stages (stage 0 for compile-time, stage 1 for runtime, etc.)
2. **Type-Safe Stage Transitions**: Compile-time guarantees for valid stage transitions through quotation and splicing
3. **Stage-Aware Representations**: AST nodes carry stage information
4. **Multi-Stage Programming**: Support for arbitrary levels of staging for code generation

### Semantic Analysis Goals

1. **Modular Design**: Each analysis component should be independent and composable
2. **Code Reuse**: Share common infrastructure across different analyses
3. **Incremental Analysis**: Support analyzing partial programs and incremental updates
4. **Error Recovery**: Continue analysis after encountering errors
5. **Extensibility**: Easy to add new analysis passes

## Background: Compile Stage Level Motivation

Traditional single-stage designs suffer from:
- No distinction between compile-time and runtime code
- Limited metaprogramming capabilities
- Lack of type safety for stage-specific operations
- Difficulty in implementing hygienic macros

MacroCaml's approach demonstrates the benefits of:
- **Stage Separation**: Clear distinction between compile-time and runtime code
- **Typed Staging**: Type safety across stages
- **Quotation-based Staging**: Code can be quoted and spliced across stages
- **Compile-time Evaluation**: Computations execute during compilation

## Architecture

### Stage-Aware Compilation Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Stage-Aware Compilation Pipeline                  │
├─────────────────────────────────────────────────────────────────────┤
│  Source → Parse → Stage Analysis → Type Check → Stage Eval → Output │
│            ↓           ↓              ↓            ↓                 │
│        Parsing    Stage Assign    Type+Stage    Compile-time        │
│                   + Desugar       Inference      Evaluation         │
└─────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────┐
│                       Shared Infrastructure                          │
├─────────────────────────────────────────────────────────────────────┤
│  • Symbol Tables    • Type Registry    • Error Collector            │
│  • Scope Manager    • Type Unifier     • Diagnostics               │
│  • Stage Manager    • Transform Traits • Analysis Context           │
└─────────────────────────────────────────────────────────────────────┘
```

### AST Stage-Based Separation

Following MacroCaml's techniques, the AST is separated based on compile stage level values. Each stage level has its own AST representation:

#### Stage 0 AST - Compile-Time Code
- Code that executes during compilation
- Macro definitions and expansions
- Type-level computations
- Generates code for Stage 1

```rust
pub mod stage0_ast {
    use crate::Position;
    
    // AST for compile-time evaluation
    pub struct Module {
        pub position: Position,
        pub name: String,
        pub macros: Vec<MacroDefinition>,
        pub compile_time_values: Vec<CompileTimeBinding>,
        pub generated_code: Vec<stage1_ast::Item>, // Code generated for stage 1
    }
    
    pub struct MacroDefinition {
        pub name: String,
        pub params: Vec<MacroParam>,
        pub body: Expression, // Evaluates to stage 1 code
    }
    
    pub enum Expression {
        Quote(Box<stage1_ast::Expression>), // Quote stage 1 code
        Splice(Box<Expression>), // Splice stage 0 value
        MacroCall { name: String, args: Vec<Expression> },
        // ... other compile-time expressions
    }
}
```

#### Stage 1 AST - Runtime Code
- Normal program code that executes at runtime
- Result of stage 0 evaluation and macro expansion
- Can contain quotes for stage 2 code generation

```rust
pub mod stage1_ast {
    use crate::Position;
    
    // AST for runtime code
    pub struct Module {
        pub position: Position,
        pub name: String,
        pub items: Vec<Item>,
    }
    
    pub enum Expression {
        Literal(Value),
        Variable(String),
        FunctionCall { func: Box<Expression>, args: Vec<Expression> },
        Quote(Box<stage2_ast::Expression>), // Quote for next stage
        // ... normal runtime expressions
    }
}
```

#### Stage n AST - Multi-Stage Code Generation
- For stages beyond 1, supporting arbitrary levels of code generation
- Each stage n generates code for stage n+1

```rust
pub mod stage_n_ast {
    use crate::Position;
    
    pub struct StagedExpression<const N: usize> {
        pub expr: Expression<N>,
        pub position: Position,
    }
    
    pub enum Expression<const N: usize> {
        Quote(Box<Expression<{N + 1}>>), // Quote for next stage
        Splice(Box<Expression<{N - 1}>>), // Splice from previous stage
        // ... stage-specific operations
    }
}
```

### Stage Calculation and Analysis

Following MacroCaml's approach, stage levels are calculated through analysis:

#### Stage Level Calculation
```rust
pub struct StageCalculator {
    current_stage: Stage,
    symbol_stages: HashMap<SymbolId, Stage>,
}

impl StageCalculator {
    pub fn calculate_expression_stage(&mut self, expr: &Expression) -> Stage {
        match expr {
            Expression::Quote(inner) => {
                // Quoted expressions are at next stage
                self.current_stage.next()
            }
            Expression::Splice(inner) => {
                // Spliced expressions must be from previous stage
                self.current_stage.prev()
                    .expect("Cannot splice at stage 0")
            }
            Expression::Variable(name) => {
                // Look up variable's stage
                self.lookup_symbol_stage(name)
            }
            // ... other cases
        }
    }
}
```

#### Stage Separation Process
1. **Parse**: Create initial AST without stage information
2. **Stage Analysis**: Calculate stage levels for all expressions
3. **Stage Separation**: Split AST into stage-specific representations
4. **Stage 0 Evaluation**: Execute compile-time code
5. **Code Generation**: Generate stage 1 AST from stage 0 results

### Stage-Aware Type System

Types in the system are stage-aware, ensuring type safety across compile-time boundaries:

```rust
pub mod stage_types {
    use crate::analysis::{Stage, TypeId};
    
    pub enum StagedType {
        // Regular types exist at specific stages
        Base { ty: TypeId, stage: Stage },
        
        // Code types represent quoted expressions
        Code { inner: Box<StagedType>, from_stage: Stage },
        
        // Stage-polymorphic types
        StagePolymorphic { 
            param: StageParam,
            inner: Box<StagedType>,
        },
    }
    
    // Type checking ensures stage consistency
    pub fn check_splice(
        expr_type: &StagedType,
        current_stage: Stage,
    ) -> Result<TypeId, StageError> {
        match expr_type {
            StagedType::Code { inner, from_stage } 
                if *from_stage == current_stage.prev()? => {
                // Valid splice
                Ok(inner.base_type())
            }
            _ => Err(StageError::InvalidSplice),
        }
    }
}
```

### Stage 0 Evaluation Engine

The compile-time evaluation engine executes stage 0 code:

```rust
pub struct Stage0Evaluator {
    symbol_table: SymbolTable,
    type_registry: TypeRegistry,
    value_env: HashMap<SymbolId, CompileTimeValue>,
}

impl Stage0Evaluator {
    pub fn eval_module(
        &mut self,
        module: &stage0_ast::Module,
    ) -> Result<stage1_ast::Module, EvalError> {
        // Evaluate all compile-time bindings
        for binding in &module.compile_time_values {
            let value = self.eval_expression(&binding.value)?;
            self.value_env.insert(binding.id, value);
        }
        
        // Generate stage 1 code
        let mut items = Vec::new();
        for generated in &module.generated_code {
            items.push(self.expand_template(generated)?);
        }
        
        Ok(stage1_ast::Module {
            position: module.position.clone(),
            name: module.name.clone(),
            items,
        })
    }
    
    pub fn eval_macro_call(
        &mut self,
        name: &str,
        args: Vec<CompileTimeValue>,
    ) -> Result<stage1_ast::Expression, EvalError> {
        // Look up macro definition
        // Bind arguments
        // Evaluate macro body
        // Return generated stage 1 code
        todo!()
    }
}
```

### Stage Transformation Infrastructure

```rust
// Transformation between stage representations
pub trait StageTransform<const FROM: usize, const TO: usize> {
    type Error;
    type Context;
    
    fn transform(
        from: StageAst<FROM>,
        ctx: &mut Self::Context,
    ) -> Result<StageAst<TO>, Self::Error>;
}

// Stage separation from unified AST
pub struct StageSeparator {
    stage_calculator: StageCalculator,
}

impl StageSeparator {
    pub fn separate(
        &mut self,
        ast: UnifiedAst,
    ) -> Result<StageSeparatedAst, StageError> {
        // Calculate stages for all nodes
        let stages = self.stage_calculator.calculate(&ast)?;
        
        // Separate into stage-specific ASTs
        let stage0 = self.extract_stage_0(&ast, &stages)?;
        let stage1_template = self.extract_stage_1_template(&ast, &stages)?;
        
        Ok(StageSeparatedAst {
            stage0,
            stage1_template,
        })
    }
}
```

### Stage-Aware Infrastructure

```rust
// Stage tracking throughout compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stage(pub u32);

impl Stage {
    pub const COMPILE_TIME: Stage = Stage(0);
    pub const RUNTIME: Stage = Stage(1);
    
    pub fn next(self) -> Stage {
        Stage(self.0 + 1)
    }
    
    pub fn prev(self) -> Result<Stage, StageError> {
        if self.0 > 0 {
            Ok(Stage(self.0 - 1))
        } else {
            Err(StageError::NoPreviousStage)
        }
    }
}

// Stage-aware symbol table
impl SymbolTable {
    pub fn declare_staged(
        &mut self,
        name: String,
        stage: Stage,
        kind: SymbolKind,
    ) -> Result<SymbolId, SymbolError> {
        let symbol = Symbol {
            name,
            kind,
            stage,
            // ...
        };
        self.declare_symbol(symbol)
    }
    
    pub fn lookup_at_stage(
        &self,
        name: &str,
        current_stage: Stage,
    ) -> Option<SymbolId> {
        // Symbols from earlier stages (closer to 0) are visible at later stages
        // E.g., at stage 2, can see symbols from stages 0, 1, and 2
        // This enables compile-time values to be used at runtime
        self.lookup(name)
            .filter(|id| {
                let symbol_stage = self.get_symbol(*id).map(|s| s.stage);
                matches!(symbol_stage, Some(s) if s <= current_stage)
            })
    }
    
    pub fn lookup_all_visible_stages(
        &self,
        name: &str,
        current_stage: Stage,
    ) -> Vec<(SymbolId, Stage)> {
        // Find all symbols with this name at visible stages
        // Useful for error messages about ambiguous symbols
        self.lookup_all(name)
            .into_iter()
            .filter_map(|id| {
                let symbol = self.get_symbol(id)?;
                if symbol.stage <= current_stage {
                    Some((id, symbol.stage))
                } else {
                    None
                }
            })
            .collect()
    }
}
```

### Component 1: Symbol Collection and Name Resolution

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

### Component 2: Type System Infrastructure

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

### Component 3: Advanced Analysis

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

1. **Type Safety**: How well does the approach prevent stage-related errors?
2. **Modularity**: How easy is it to add new stages or modify existing ones?
3. **Performance**: What is the runtime and compile-time overhead?
4. **Usability**: How complex is the API for compiler developers?
5. **Extensibility**: How well does it support future language features?

### Design Decision: Hybrid Approach

After evaluating multiple design options, the recommended approach follows MacroCaml's staged compilation model:

1. **Stage-based AST separation** for clarity and type safety
2. **Trait-based separation** for code reuse and flexibility
3. **Shared infrastructure** for common functionality

This hybrid provides:
- Clear stage separation with distinct AST types per compile stage level
- Trait-based common functionality
- Type-safe transformations
- Extensible architecture
- Optimal balance of type safety, modularity, and usability

### Implementation Strategy

The implementation combines both AST stage separation and semantic analysis into a unified pipeline:

#### Phase 1: Foundation and Stage Infrastructure (Week 1-2)
1. Define core stage types and transformation framework
2. Implement `SymbolTable` and `Scope` management
3. Create `ErrorCollector` and diagnostic infrastructure
4. Set up basic module structure for stage-specific ASTs

#### Phase 2: Parse AST and Stage Analysis (Week 3-4)
1. Implement unified AST parsing
2. Add stage calculation and separation
3. Build basic `NameResolver` visitor
4. Update parser to generate parse_ast nodes

#### Phase 3: Type System and Stage-Aware Types (Week 5-6)
1. Implement `TypeRegistry` and core type representations
2. Design stage-aware type system
3. Create `TypeChecker` visitor as Transform trait
4. Add type inference for simple cases

#### Phase 4: Stage Evaluation and Code Generation (Week 7-8)
1. Implement stage 0 evaluator
2. Generate stage 1 code from stage 0 evaluation
3. Add control flow analysis
4. Basic ownership/lifetime checking for pointer types

#### Phase 5: Integration and Optimization (Week 9-10)
1. Create analysis pipeline orchestrator
2. Update visitor pattern for stage-specific traversals
3. Add incremental analysis support
4. Performance testing and optimization

#### Phase 6: Documentation and Testing (Week 11-12)
1. Comprehensive documentation for stage system
2. Migration guide for existing code
3. Extensive test coverage for all phases
4. Integration tests for complete pipeline

## Stage Visibility Example

```patina
// Stage 0 (compile-time)
const TABLE_SIZE: i32 = 256;  // Visible at all stages >= 0

macro generate_lookup() {
    // This runs at stage 0, generates stage 1 code
    let table = Array::new(TABLE_SIZE);
    <[~table]>  // Quote stage 1 array using stage 0 value
}

// Stage 1 (runtime)
fn main() {
    let size = TABLE_SIZE;  // Can use stage 0 constant
    let lookup = generate_lookup();  // Macro expansion happens at stage 0
    
    // This would be an error - can't access future stages:
    // let future = ~stage2_value;  // Error: stage2_value not visible at stage 1
}
```

## Usage Example

```rust
use patina::analysis::{analyze_pipeline, PipelineOptions};
use patina::analysis::{Stage, StageSeparator, Stage0Evaluator};

// Parse source into unified AST
let source = std::fs::read_to_string("example.pat")?;
let unified_ast = parse_to_ast(&source)?;

// Option 1: Run complete pipeline
let options = PipelineOptions {
    type_check: true,
    check_ownership: true,
    optimize: true,
};

let result = analyze_pipeline(unified_ast, options)?;

// Option 2: Run stages individually for more control
let mut ctx = AnalysisContext::new();

// Separate AST by stages
let separated = StageSeparator::new()
    .separate(unified_ast, &mut ctx)?;

// Evaluate stage 0 (compile-time)
let stage1_ast = Stage0Evaluator::new()
    .eval_module(&separated.stage0, &mut ctx)?;

// Access stage-specific information
let symbols = ctx.symbol_table();
let types = ctx.type_registry();
let diagnostics = ctx.diagnostics();

// Stage 1 is now ready for runtime compilation
if let Some(main_fn) = stage1_ast.find_function("main") {
    println!("main function type: {:?}", main_fn.signature());
}

// Generate final code from stage 1
let output = CodeGenerator::new()
    .generate(stage1_ast, &mut ctx)?;
```

## Benefits of This Design

### Stage Separation Benefits

1. **Clear Stage Boundaries**: Each compile-time stage has distinct responsibilities
2. **Type-Safe Stage Transitions**: Invalid stage transitions caught at compile time
3. **Powerful Metaprogramming**: Arbitrary compile-time computation via stage 0
4. **Hygienic Macros**: Stage separation ensures proper macro hygiene
5. **Multi-Stage Programming**: Support for code generation at multiple levels

### Semantic Analysis Benefits

1. **Modularity**: Each analysis phase is independent and composable
2. **Reusability**: Shared infrastructure reduces duplication across phases
3. **Extensibility**: New analyses can be added as visitors or transformers
4. **Performance**: Supports incremental analysis and caching
5. **Error Recovery**: Can continue analysis after encountering errors
6. **Type Safety**: Leverages Rust's type system for correctness

### Combined Architecture Benefits

1. **Clean Separation of Concerns**: Compile-time and runtime code are clearly separated
2. **Flexible Pipeline**: Can evaluate stages individually or as a complete pipeline
3. **Better Error Messages**: Stage-specific context enables precise diagnostics
4. **Easier Testing**: Each stage can be tested in isolation
5. **MacroCaml Compatibility**: Architecture follows proven staging techniques

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