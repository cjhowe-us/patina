# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with snapshot review (interactive)
cargo insta test --review

# Accept all snapshot changes without review
cargo insta test --accept

# Run tests without updating snapshots
cargo insta test --check

# Accept snapshots from last test run
cargo insta accept

# Run a specific test
cargo test test_name

# Run tests in a specific module
cargo test module_name::
```

### Linting

```bash
# Run clippy (Rust linter)
cargo clippy

# Fix auto-fixable warnings
cargo fix
```

## Architecture Overview

Patina is a systems programming language with **planned phase-separated compilation**:

### Current Implementation

1. **Parser**: Complete, generates unified AST
2. **Semantic Analysis**: Basic type checking and symbol resolution
3. **Stage Infrastructure**: Foundation types defined but not integrated

### Planned Architecture

See `docs/plans/phase-separation/` for the complete stage-based compilation design.

### Core Components

1. **Parser**: Uses the Pest parser generator
   - Grammar defined in `src/grammar.pest`
   - Parser implementation in `src/parser.rs`
   - Converts source text into AST nodes

2. **AST (Abstract Syntax Tree)**:
   - Implementation in `src/ast/`
   - Organized by node categories: statements, expressions, atoms, types, values
   - All nodes implement the `SyntaxNode` trait

3. **Semantic Analysis**:
   - Implementation in `src/analysis/`
   - Components: arena, context, diagnostics, stage, symbol_table, types
   - Stage-aware analysis and validation

4. **Visitor Pattern**:
   - Original visitor implementation in `src/visitor.rs`
   - Enhanced visitor pattern in `src/ast/walk.rs`
   - Provides tree traversal without match statements
   - Used for analysis, transformation, and code generation

### Stage System

- **Stage 0**: Compile-time execution
- **Stage 1**: Runtime (default)
- **Stage n**: Code generated at Stage n-1

### Type System

The language supports multiple pointer ownership models:

- `*own` - Owned pointers
- `*raw` - Raw pointers
- `*mut` - Mutable pointers
- `*` - Read only pointers

### Testing Strategy

Tests are organized into:

- Test cases stored in separate JSON data files
- Use snapshot testing for assertions
- Error case testing to verify proper error handling

## Documentation Structure

Comprehensive documentation is available in `docs/`:

- **User Documentation** (`docs/user/`) - Getting started guide
- **Developer Documentation** (`docs/developer/`) - Architecture and testing
- **Plans** (`docs/plans/`) - Implementation roadmaps, especially phase separation
- **Designs** (`docs/designs/`) - Architectural designs
- **RFCs** (`docs/rfcs/`) - Problem statements and improvement proposals
- **Status** (`docs/status/`) - Current implementation progress

See `docs/README.md` for detailed navigation.