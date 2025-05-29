# Getting Started with Patina

This guide will help you build, test, and start developing with the Patina programming language.

## Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- Git

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/patina.git
cd patina
```

2. Build the project:
```bash
cargo build
```

3. Run tests to verify installation:
```bash
cargo test
```

## Project Structure

```
patina/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library root
│   ├── parser.rs        # Parser implementation
│   ├── grammar.pest     # Language grammar
│   ├── ast/             # AST definitions
│   ├── analysis/        # Semantic analysis
│   │   ├── arena.rs     # Memory arena
│   │   ├── context.rs   # Analysis context
│   │   ├── diagnostics.rs # Error reporting
│   │   ├── stage.rs     # Stage analysis
│   │   ├── symbol_table.rs # Symbol resolution
│   │   └── types.rs     # Type system
│   └── visitor.rs       # Visitor pattern
├── test_cases/          # Test data files
├── docs/               # Documentation
└── Cargo.toml          # Project configuration
```

## Development Workflow

### Building

```bash
# Debug build (fast compilation)
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

### Running the Compiler

```bash
# Run with a source file
cargo run -- input.pat

# Run with additional options
cargo run -- --help
```

### Code Quality

```bash
# Run linter
cargo clippy

# Fix auto-fixable warnings
cargo fix

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

## Understanding the Codebase

### Key Modules

1. **Parser** (`src/parser.rs`): Converts source text to AST
2. **AST** (`src/ast/`): Abstract syntax tree definitions
3. **Analysis** (`src/analysis/`): Semantic analysis and type checking
4. **Grammar** (`src/grammar.pest`): Language syntax rules

### Important Files

- `CLAUDE.md`: AI assistant instructions
- `readme.md`: Project overview
- Test files in `test_cases/`: Example Patina code

## Writing Patina Code

### Basic Syntax

```patina
// Variable binding
let x = 42
let mut y = "hello"

// Functions
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Control flow
if x > 0 {
    print("positive")
} else {
    print("non-positive")
}
```

### Stage Annotations

```patina
// Compile-time computation
@[stage(0)]
let size = 100

// Runtime array with compile-time size
let data: [i32; $(size)] = [0; $(size)]
```

### Type System

```patina
// Pointer types
let owned: *own i32 = alloc(42)
let borrowed: *i32 = &value
let mutable: *mut i32 = &mut value
let raw: *raw i32 = unsafe_ptr
```

## Common Tasks

### Adding a New AST Node

1. Define the struct in appropriate `src/ast/*.rs` file
2. Implement `SyntaxNode` trait
3. Add parsing logic in grammar
4. Update tests

### Adding a New Analysis Pass

1. Create new file in `src/analysis/`
2. Implement visitor using walk functions
3. Integrate with analysis context
4. Add tests

### Debugging Parser Issues

1. Use `RUST_LOG=debug` for detailed output
2. Check grammar rules in `src/grammar.pest`
3. Examine test snapshots for expected output

## Next Steps

- Explore [Testing](../developer/testing.md) to understand the test infrastructure
- Read [Architecture Overview](../developer/architecture/overview.md) for system design
- Check [Current Status](../status/current.md) for ongoing work
- Study [Stage Levels](../designs/stage-levels.md) for metaprogramming

## Getting Help

- Check existing tests for examples
- Review documentation in `docs/`
- Look at snapshot tests for expected behavior
- Examine error messages in `src/analysis/diagnostics.rs`