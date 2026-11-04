# AGENTS.md

This file provides guidance to coding agents working in this repository.

## Development Commands

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run tests in a specific module
cargo test module_name::
```

There is no snapshot testing; `insta` is not a dependency (finding TEST-9). See `docs/developer/testing.md`.

### Formatting and Linting

```bash
# Check formatting, as CI does
cargo fmt --all --check

# Run clippy with CI's flags
cargo clippy --all-targets --all-features -- -D warnings

# Fix auto-fixable warnings
cargo fix
```

## Architecture Overview

Patina is a systems programming language with **planned phase-separated compilation**:

### Current Implementation

1. **Parser**: Partial. Parses most of the grammar into the AST, but drops or replaces some constructs (for example grouped expressions, type syntax, and annotations) and panics on others such as `struct` (PARSE findings)
2. **Semantic Analysis**: Standalone symbol table, type, and diagnostic types; not connected to the parser
3. **Stage Infrastructure**: Foundation types defined but not integrated

### Planned Architecture

The active plan is `docs/plans/comptime-reflection/` and covers comptime metaprogramming, static reflection, and incremental compilation. The compiler IR must be a Regionalized Value State Dependence Graph (RVSDG), represented with `hecs` entities and components. The IR is the only structure required to use `hecs`; the AST is a plain owned tree (see below). The removed phase-separation plan's unfinished work is recorded in the active plan's carry-over table. Work on it one step at a time with the `work-plan` skill (`/work-plan comptime-reflection`). Known defects are tracked by ID in `docs/plans/comptime-reflection/findings.md`.

### Core Components

1. **Parser**: Uses the Pest parser generator
   - Grammar defined in `src/grammar.pest`
   - Parser implementation in `src/parser.rs`
   - Converts source text into AST nodes

2. **AST (Abstract Syntax Tree)**:
   - Implementation in `src/ast/` (node types in `src/ast/nodes.rs`)
   - A plain owned Rust tree: each node owns its children inline. It is not stored in `hecs`
   - `Ast` also holds a vestigial `hecs::World` side table keyed by per-node entities; the parser stores only `Position` in it (finding AST-9)
   - See `docs/developer/ast.md`

3. **Semantic Analysis**:
   - Implementation in `src/analysis/`
   - Components: arena, context, diagnostics, stage, symbol_table, types
   - Stage-aware analysis and validation

4. **Visitor Pattern**:
   - `Visitor` trait and `walk_*` functions in `src/ast/visitor.rs`
   - Preorder traversal of the node tree
   - See `docs/developer/architecture/visitor-pattern.md`

### Stage System

The existing code models numbered stages (`analysis::Stage(u32)`):

- **Stage 0**: Compile-time execution
- **Stage 1**: Runtime (default)
- **Stage n**: Code generated at Stage n-1

"Stage" is a provisional legacy term. The plan splits it into a function's inferred comptime/runtime **mode**, an item's **sealed** (fully reified) state, and the **reification order** between items, and will rename it (finding ANL-10). Do not build new features on the stage APIs.

Patina needs deeper static analysis than most languages, beyond even Rust: ownership checking for every pointer kind, mode inference, comptime-computed types, generated-code hygiene, and effects. Each analysis must be an incremental query (constraint C-9, finding ANL-11).

### Type System

The language supports multiple pointer ownership models:

- `*own` - Owned pointers
- `*raw` - Raw pointers
- `*mut` - Mutable pointers
- `*` - Read only pointers

### Testing Strategy

Tests are organized into:

- Parser test cases stored as RON files in `test_cases/parser/valid/` and `test_cases/parser/errors/`
- Most valid fixtures only check that parsing succeeds; error fixtures only check that it fails (findings TEST-1, TEST-6)
- Unit tests for the AST and analysis types build their inputs by hand
- See `docs/developer/testing.md`

## Documentation Structure

Comprehensive documentation is available in `docs/`:

- **Developer Documentation** (`docs/developer/`) - Getting started, architecture, AST, and testing
- **Plans** (`docs/plans/`) - The active comptime-reflection plan and shared plan conventions
- **Designs** (`docs/designs/`) - Architectural designs, to be created by Step 1 of the active plan
- **RFCs** (`docs/rfcs/`) - Problem statements and improvement proposals
- **Status** (`docs/status/`) - Current implementation progress

See `docs/README.md` for detailed navigation.