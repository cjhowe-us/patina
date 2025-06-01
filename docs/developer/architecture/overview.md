# Patina Architecture Overview

## Current Architecture

Patina implements a component-based AST using Entity-Component-System (ECS) architecture, with separate semantic analysis infrastructure ready for integration.

### Core Components (Implemented)

- **Parser**: Complete AST generation with Pest
- **Component-Based AST**: ECS architecture using `hecs`
- **Semantic Analysis**: Infrastructure complete but not integrated
- **Stage System**: Types defined, awaiting integration

## Planned Pipeline (See docs/plans/phase-separation)

Future stage-based compilation phases:

1. **Parse Phase** (Currently implemented)
2. **Stage Analysis** (Planned)
3. **Stage Separation** (Planned)
4. **Stage 0 Evaluation** (Planned)
5. **Code Generation** (Planned)

## Core Components

### Parser
- **Location**: `src/parser.rs`, `src/grammar.pest`
- **Purpose**: Convert text to AST
- **Design**: Pest parser generator
- **Output**: Strongly-typed AST nodes

### Component-Based AST
- **Location**: `src/ast/`
- **Architecture**: Entity-Component-System (ECS)
- **Structure**:
  - `mod.rs` - Core `Ast<C>` generic type
  - `nodes.rs` - All AST node definitions
  - `components.rs` - Component types
  - `visitor.rs` - Visitor patterns
  - `printer.rs` - Debug utilities
- **Features**:
  - Generic components without boxing
  - Efficient entity queries
  - Type-safe at compile time

### Semantic Analysis
- **Location**: `src/analysis/`
- **Components**:
  - `arena.rs` - Memory management
  - `context.rs` - Analysis context
  - `diagnostics.rs` - Error reporting
  - `id.rs` - Type-safe identifiers
  - `stage.rs` - Stage analysis
  - `symbol_table.rs` - Symbol resolution
  - `types.rs` - Type checking

### Stage System
- **Stage 0**: Compile-time execution
- **Stage 1**: Runtime (default)
- **Stage n**: Code generated at Stage n-1

### Type System
**Ownership-based pointers**:
- `*own` - Owned pointers
- `*raw` - Raw pointers
- `*mut` - Mutable references
- `*` - Immutable references

## Design Patterns

### Visitor Pattern
- **Traditional**: Node-specific visitor methods
- **Generic**: Component-based traversal
- **Filter**: Query entities by component
- See: `src/ast/visitor.rs`

### Arena Allocation
- Efficient memory management
- Reduced allocations
- Better cache locality
- See: `src/analysis/arena.rs`

### Trait-Based Design
- `SyntaxNode` trait for all AST nodes
- Generic algorithms over node types
- Clean separation of concerns

## Key Benefits

### Phase Separation Advantages
- **Modularity**: Independent phases
- **Testability**: Isolated testing
- **Performance**: Stage-specific optimization
- **Clarity**: Clear data flow

### Stage System Benefits
- **Metaprogramming**: Powerful compile-time computation
- **Zero-cost**: No runtime overhead
- **Type Safety**: Stage checking prevents errors
- **Extensibility**: Add new stages easily

## Module Organization

```
src/
├── main.rs          # CLI entry (parser only)
├── lib.rs           # Library root
├── parser.rs        # Parser implementation
├── grammar.pest     # Language grammar
├── ast/             # Component-based AST
│   ├── mod.rs       # Core AST structure
│   ├── nodes.rs     # Node definitions
│   ├── components.rs # ECS components
│   └── visitor.rs   # Visitor patterns
└── analysis/        # Semantic analysis (not integrated)
    ├── arena.rs     # Memory management
    ├── context.rs   # Analysis context
    └── types.rs     # Type system
```

## Extension Points

### Adding Phases
- Implement phase interface
- Define input/output types
- Integrate into pipeline

### New AST Nodes
- Implement `SyntaxNode` trait
- Add to appropriate module
- Update grammar if needed

### Analysis Passes
- Use visitor pattern
- Add to analysis pipeline
- Maintain phase boundaries