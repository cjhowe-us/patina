# AST Separation Design for Patina

## Overview

This document explores different approaches for splitting the Patina AST into separate trees based on compilation stages, inspired by MacoCaml's phase separation techniques. The goal is to create a more modular, type-safe compilation pipeline where each stage has its own specialized AST representation.

## Background: MacoCaml's Approach

MacoCaml introduces a novel combination of macros with phase separation and quotation-based staging. Key concepts include:

1. **Phase Separation**: Clear distinction between compile-time and runtime code
2. **Typed Source Calculus**: The "maco" calculus ensures type safety across phases
3. **Quotation-based Staging**: Code can be quoted and spliced across phases
4. **Compile-time Evaluation**: Macros execute during compilation within top-level splices

## Current Patina AST Structure

The current Patina AST is a single unified tree structure with:
- All node types in one hierarchy
- Shared `SyntaxNode` trait for traversal
- No explicit separation of compilation phases
- Visitor pattern for generic tree operations

## Design Options

### Option 1: Phase-Based AST Hierarchy

Create separate AST types for each compilation phase:

```rust
// Phase 1: Raw Parse Tree
mod parse_tree {
    pub enum ParseNode {
        Module(ParseModule),
        Statement(ParseStatement),
        Expression(ParseExpression),
        // Raw syntactic elements
    }
}

// Phase 2: Desugared AST
mod desugared_ast {
    pub enum DesugaredNode {
        Module(DesugaredModule),
        Function(DesugaredFunction),
        // No syntactic sugar, expanded macros
    }
}

// Phase 3: Typed AST
mod typed_ast {
    pub enum TypedNode {
        Module(TypedModule),
        Function(TypedFunction),
        // With type annotations
    }
}

// Phase 4: IR Tree
mod ir_tree {
    pub enum IRNode {
        BasicBlock(BasicBlock),
        Instruction(Instruction),
        // Low-level representation
    }
}
```

**Advantages:**
- Clear separation of concerns
- Type-safe phase transitions
- Optimized representations per phase
- Easy to add phase-specific operations

**Disadvantages:**
- Code duplication across phases
- Complex transformation functions
- Loss of unified traversal

### Option 2: Parameterized AST with Phantom Types

Use phantom types to track compilation phases:

```rust
use std::marker::PhantomData;

// Phase markers
struct Parsed;
struct Desugared;
struct Typed;
struct Lowered;

// Parameterized AST
pub struct AST<Phase> {
    root: Module<Phase>,
    _phantom: PhantomData<Phase>,
}

pub struct Module<Phase> {
    name: String,
    items: Vec<Item<Phase>>,
    _phantom: PhantomData<Phase>,
}

// Phase-specific data as associated types
trait CompilationPhase {
    type TypeInfo;
    type Metadata;
}

impl CompilationPhase for Parsed {
    type TypeInfo = ();
    type Metadata = SourceLocation;
}

impl CompilationPhase for Typed {
    type TypeInfo = Type;
    type Metadata = (SourceLocation, TypeInfo);
}
```

**Advantages:**
- Single AST structure with phase safety
- Gradual enrichment of nodes
- Type-safe phase transitions
- Reduced code duplication

**Disadvantages:**
- Complex type signatures
- Limited flexibility for phase-specific structures
- Phantom types can be confusing

### Option 3: Trait-Based Phase Separation

Define traits for each compilation phase:

```rust
trait ParsePhase: SyntaxNode {
    fn source_location(&self) -> &SourceLocation;
}

trait TypePhase: ParsePhase {
    fn type_info(&self) -> &TypeInfo;
}

trait OptimizationPhase: TypePhase {
    fn optimization_hints(&self) -> &OptHints;
}

// Concrete implementations
struct ParsedFunction {
    location: SourceLocation,
    name: String,
    body: ParsedBlock,
}

struct TypedFunction {
    parsed: ParsedFunction,
    return_type: Type,
    param_types: Vec<Type>,
}

impl ParsePhase for TypedFunction {
    fn source_location(&self) -> &SourceLocation {
        &self.parsed.location
    }
}

impl TypePhase for TypedFunction {
    fn type_info(&self) -> &TypeInfo {
        &self.type_info
    }
}
```

**Advantages:**
- Incremental addition of phase data
- Maintains unified traversal capability
- Clear phase dependencies
- Extensible design

**Disadvantages:**
- Trait bounds can become complex
- Runtime overhead from trait objects
- Less clear phase separation

### Option 4: Staging with Quotation Support

Implement a staging system similar to MacoCaml:

```rust
// Quotation type for staged code
pub enum Quote<T> {
    Lift(T),                    // Lift value to next stage
    Splice(Box<dyn Fn() -> T>), // Splice computation
    Code(String),               // Raw code representation
}

// Staged AST nodes
pub enum StagedExpr {
    // Runtime expressions
    Runtime(RuntimeExpr),
    
    // Compile-time expressions
    CompileTime(CompileTimeExpr),
    
    // Quoted expressions (for next stage)
    Quote(Box<Quote<StagedExpr>>),
    
    // Spliced expressions (from previous stage)
    Splice(SpliceRef),
}

// Stage tracking
pub struct StageContext {
    current_stage: u32,
    bindings: HashMap<String, (u32, Value)>, // stage -> value
}
```

**Advantages:**
- Supports metaprogramming
- Clear stage boundaries
- Enables compile-time code generation
- Similar to proven MacoCaml approach

**Disadvantages:**
- Complex implementation
- Requires careful stage tracking
- Learning curve for users

### Option 5: Modular AST Components

Separate AST into modular components that can be composed:

```rust
// Core AST traits
trait ASTNode {
    fn id(&self) -> NodeId;
}

trait HasPosition: ASTNode {
    fn position(&self) -> &Position;
}

trait HasType: ASTNode {
    fn node_type(&self) -> &Type;
}

// Component storage
struct ASTComponents {
    positions: HashMap<NodeId, Position>,
    types: HashMap<NodeId, Type>,
    scopes: HashMap<NodeId, ScopeId>,
    // ... other components
}

// Phase-specific views
struct ParseView<'a> {
    nodes: &'a NodeStorage,
    positions: &'a HashMap<NodeId, Position>,
}

struct TypedView<'a> {
    parse: ParseView<'a>,
    types: &'a HashMap<NodeId, Type>,
}
```

**Advantages:**
- Flexible composition
- Memory efficient
- Easy to add new components
- Clear separation of concerns

**Disadvantages:**
- Indirect access patterns
- Complex lifetime management
- Loss of type safety without careful design

## Evaluation and Recommendation

### Evaluation Criteria

1. **Type Safety**: How well does the approach prevent phase-related errors?
2. **Modularity**: How easy is it to add new phases or modify existing ones?
3. **Performance**: What is the runtime and compile-time overhead?
4. **Usability**: How complex is the API for compiler developers?
5. **Extensibility**: How well does it support future language features?

### Evaluation Results

| Approach | Type Safety | Modularity | Performance | Usability | Extensibility | Overall |
|----------|------------|------------|-------------|-----------|---------------|---------|
| Option 1 | Excellent  | Good       | Excellent   | Fair      | Good          | Good    |
| Option 2 | Excellent  | Fair       | Good        | Poor      | Fair          | Fair    |
| Option 3 | Good       | Excellent  | Fair        | Good      | Excellent     | Good    |
| Option 4 | Good       | Good       | Fair        | Poor      | Excellent     | Good    |
| Option 5 | Fair       | Excellent  | Good        | Fair      | Excellent     | Good    |

### Recommended Approach: Hybrid of Options 1 and 3

The best approach combines the clarity of phase-based AST hierarchy (Option 1) with the flexibility of trait-based separation (Option 3):

```rust
// Phase-specific AST modules
pub mod parse_ast {
    use super::traits::*;
    
    pub struct Module {
        pub position: Position,
        pub name: String,
        pub items: Vec<Item>,
    }
    
    impl ParsePhase for Module {
        fn position(&self) -> &Position { &self.position }
    }
}

pub mod typed_ast {
    use super::traits::*;
    use super::parse_ast;
    
    pub struct Module {
        pub parsed: parse_ast::Module,
        pub exports: HashMap<String, Type>,
        pub type_env: TypeEnvironment,
    }
    
    impl ParsePhase for Module {
        fn position(&self) -> &Position { self.parsed.position() }
    }
    
    impl TypedPhase for Module {
        fn type_env(&self) -> &TypeEnvironment { &self.type_env }
    }
}

// Transformation traits
pub trait Transform<From, To> {
    type Error;
    fn transform(from: From, ctx: &mut Context) -> Result<To, Self::Error>;
}

// Phase transition functions
impl Transform<parse_ast::Module, typed_ast::Module> for TypeChecker {
    type Error = TypeError;
    
    fn transform(from: parse_ast::Module, ctx: &mut Context) -> Result<typed_ast::Module, TypeError> {
        // Type checking logic
    }
}
```

This hybrid approach provides:
- Clear phase separation with distinct AST types
- Trait-based common functionality
- Type-safe transformations
- Extensible architecture

## Implementation Plan

### Phase 1: Foundation (Week 1-2)
1. Define core traits for each compilation phase
2. Create phase marker types and transformation framework
3. Set up basic module structure for phase-specific ASTs

### Phase 2: Parse AST (Week 3-4)
1. Implement parse_ast module with current AST structures
2. Add ParsePhase trait implementations
3. Update parser to generate parse_ast nodes

### Phase 3: Typed AST (Week 5-6)
1. Design typed_ast module with type annotations
2. Implement type checker as Transform trait
3. Add TypedPhase trait implementations

### Phase 4: IR Generation (Week 7-8)
1. Design IR tree structure
2. Implement lowering from typed_ast to IR
3. Add optimization phase support

### Phase 5: Integration (Week 9-10)
1. Update visitor pattern for phase-specific traversals
2. Migrate existing passes to new architecture
3. Performance testing and optimization

### Phase 6: Documentation and Testing (Week 11-12)
1. Comprehensive documentation
2. Migration guide for existing code
3. Extensive test coverage

## Conclusion

The recommended hybrid approach provides the best balance of type safety, modularity, and usability. It allows for clear phase separation while maintaining the flexibility to share common functionality through traits. This design will support Patina's evolution as a systems programming language with advanced compile-time features.