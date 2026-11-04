# Patina

Patina is an experimental systems programming language. It is being redirected toward compile-time (`comptime`) metaprogramming and static reflection with incremental compilation, alongside memory safety through ownership types.

## Features

- **Comptime Metaprogramming and Static Reflection**: replaces generics and most special-purpose language features (planned)
- **Incremental Compilation**: per-item queries, including incremental reflection (planned)
- **RVSDG IR on `hecs`**: the planned compiler IR; the AST itself is a plain owned tree
- **Ownership Types**: `*own`, `*raw`, `*mut`, and `*` pointer modifiers (parsed, not yet checked)

## Project Status

- 🟡 **Parser**: Partial; several constructs panic or are dropped
- ✅ **AST**: Plain owned node tree with a vestigial `hecs` side table for positions
- 🟡 **Semantic Analysis Infrastructure**: Symbol tables, type registry, diagnostics (standalone)
- ❌ **Integration**: Analysis not connected to parser
- ❌ **IR and Code Generation**: Not started

See [docs/status/current.md](docs/status/current.md) and the [comptime-reflection plan](docs/plans/comptime-reflection/index.md) for details.

## Development

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests
cargo test
```

### Documentation

See the [docs/](docs/) directory for comprehensive documentation:
- [Architecture Overview](docs/developer/architecture/overview.md)
- [AST](docs/developer/ast.md)
- [Current Status](docs/status/current.md)
