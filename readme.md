# Patina

Patina is an experimental systems programming language featuring compile-time metaprogramming through stage-based compilation, memory safety through ownership types, and a component-based AST architecture.

## Features

- **Component-Based AST**: Entity-Component-System architecture for extensibility
- **Stage-Based Compilation**: Foundation for compile-time metaprogramming (planned)
- **Ownership Types**: `*own`, `*raw`, `*mut`, and `*` pointer modifiers
- **Modern Syntax**: Clean, expressive language design
- **Type Safety**: Strong static typing with planned inference

## Project Status

- ✅ **Parser**: Complete with error recovery
- ✅ **Component-Based AST**: ECS architecture implemented
- ✅ **Semantic Analysis Infrastructure**: Symbol tables, type system, diagnostics
- ❌ **Integration**: Analysis not connected to parser
- ❌ **Code Generation**: Not started

See [docs/status/current.md](docs/status/current.md) for details.

## Development

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Review snapshot changes interactively
cargo insta test --review

# Accept all snapshot changes
cargo insta test --accept

# Check without updating snapshots
cargo insta test --check
```

### Documentation

See the [docs/](docs/) directory for comprehensive documentation:
- [Architecture Overview](docs/developer/architecture/overview.md)
- [Component-Based AST](docs/developer/component-based-ast.md)
- [Current Status](docs/status/current.md)
