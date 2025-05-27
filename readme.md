# Patina

Patina is a systems programming language with a focus on memory safety, modern syntax, and powerful type system features. The project includes a parser, AST representation, and various analysis tools.

## Features

- **Module System**: Organize code into modules with explicit scoping
- **Memory Safety**: Multiple pointer types (`*own`, `*raw`, `*mut`) for different ownership semantics
- **Type System**:
  - Integer types with explicit bit widths (`i32`, `u64`, etc.)
  - Floating-point types (`f32`, `f64`)
  - Arrays with fixed and dynamic sizes
  - Function types as first-class citizens
- **Control Flow**:
  - `if`/`else` expressions
  - `case` expressions for pattern matching
  - Labeled blocks with `break` statements
  - `while` and `for` loops
- **Functions**:
  - Static and regular functions
  - First-class functions and lambdas
  - Type annotations for parameters and return values
- **Data Structures**:
  - Structs with generic type parameters
  - Enums with variant data
- **Modern Syntax**:
  - String interpolation with `\(expression)` syntax
  - Annotations/attributes with `@tag` syntax
  - Comments (line `//` and block `/* */`)
  - Expression-oriented design

## Development

### Building the code

```bash
cargo build
```

### Running unit tests

```bash
# Run all tests
cargo test

# Runs all tests and reviews any snapshot changes interactively
cargo insta test --review
```

#### Non-interactively

```bash
# Run all tests and accept any snapshot changes without interactivity
cargo insta test --accept

# Overwrites snapshots from the last test run without interactivity
cargo insta accept

# Runs the tests and does not update any snapshots, without interactivity
cargo insta test --check
```

### Directory Structure

```txt
patina/
├── src/                # Rust source code
│   ├── ast/            # AST definitions
│   ├── parser/         # Parser implementation
│   ├── visitor/        # Visitor pattern implementation
│   ├── grammar.pest    # Pest grammar definition
│   ├── lib.rs          # Library entry point
│   └── main.rs         # CLI entry point
├── test_cases/         # JSON test cases for parser testing
├── target/             # Rust build artifacts
└── Cargo.toml          # Rust project configuration
```

### Dependencies

- **pest** (2.7): Parser generator for creating the Patina parser
- **pest_derive** (2.7): Derive macros for Pest
- **serde** (1.0): Serialization/deserialization framework
- **serde_json** (1.0): JSON support for serde
- **thiserror** (1.0): Error handling with derive macros

### Development Dependencies

- **insta** (1.34): Snapshot testing library with JSON support
- **tempfile** (3.8): Temporary file handling for tests

### Testing Conventions

1. **Snapshot Tests**: Used extensively for parser and AST tests
   - Snapshots are stored in `snapshots/` directories
   - JSON format for structured data comparison
   - Named after test functions

2. **Test Organization**:
   - Unit tests are colocated with source files
   - Integration tests in `tests/` directory
   - Test cases defined in JSON files under `test_cases/`

3. **Naming Conventions**:
   - Test names use descriptive strings with underscores
   - Snapshot files include the test name

### Code Style

- Rust code follows standard `rustfmt` conventions
- Grammar rules in `grammar.pest` use snake_case
