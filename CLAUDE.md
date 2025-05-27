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

Patina is a systems programming language project implemented in Rust. The architecture follows a clear separation of concerns:

### Core Components

1. **Parser**: Uses the Pest parser generator
   - Grammar defined in `src/grammar.pest`
   - Parser implementation in `src/parser.rs`
   - Converts source text into AST nodes

2. **AST (Abstract Syntax Tree)**:
   - Implementation in `src/ast/`
   - Organized by node categories: statements, expressions, atoms, types, values
   - All nodes implement the `SyntaxNode` trait

3. **Visitor Pattern**:
   - Original visitor implementation in `src/visitor.rs`
   - Enhanced visitor pattern in `src/ast/walk.rs`
   - Provides tree traversal without match statements
   - Used for analysis, transformation, and code generation
   - See @VISITOR_PATTERN.md for detailed documentation

4. **Test Infrastructure**:
   - Snapshot testing with `insta` for parser output
   - Shared test utilities in `test_shared.rs`

### Type System

The language supports multiple pointer ownership models:
- `*own` - Owned pointers
- `*raw` - Raw pointers  
- `*mut` - Mutable pointers
- Regular pointers without modifiers

### Testing Strategy

Tests are organized into:
- Unit tests colocated with source files
- Snapshot tests for parser output and visitor patterns
- Error case testing to verify proper error handling

