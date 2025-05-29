# Patina Architecture Overview

## Current Architecture

Patina currently implements a traditional compiler pipeline with infrastructure for future stage-based phase separation.

### Core Components (Implemented)

- **Parser**: Complete AST generation
- **Semantic Analysis**: Type checking and symbol resolution
- **Foundation**: Stage types and infrastructure (not integrated)

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

### AST (Abstract Syntax Tree)
- **Location**: `src/ast/`
- **Structure**:
  - `statements.rs` - Statement nodes
  - `expressions.rs` - Expression nodes
  - `atoms.rs` - Atomic constructs
  - `types.rs` - Type representations
  - `values.rs` - Value nodes
  - `syntax_nodes.rs` - Base traits

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
- Generic AST traversal
- No match statements required
- Extensible analysis passes
- See: `src/ast/walk.rs`

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
├── main.rs          # CLI entry
├── lib.rs           # Library root
├── parser.rs        # Parser implementation
├── grammar.pest     # Language grammar
├── ast/             # AST definitions
├── analysis/        # Semantic analysis
└── visitor.rs       # Original visitor
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