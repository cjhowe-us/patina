# Patina Architecture Overview

This page describes the compiler as it is implemented today. Known defects are tracked by ID in [findings.md](../../plans/comptime-reflection/findings.md). Planned work belongs to the [comptime-reflection plan](../../plans/comptime-reflection/index.md).

## Current Pipeline

Only parsing runs today:

```text
source text
  -> PatinaParser            Pest parser generated from src/grammar.pest
  -> ParserVisitor/ParseNode builds node structs (src/parser.rs)
  -> Ast                     plain owned node tree + hecs side table holding Position (src/ast/)
  -> CLI                     prints entity and statement counts (src/main.rs)
```

`src/analysis/` compiles and has unit tests, but nothing calls it. There is no name resolution, type checking, comptime evaluation, IR, or code generation.

## Component Status

| Component | Location | Status |
|---|---|---|
| Grammar | `src/grammar.pest` | Accepts most of the surface syntax, including `struct`, `enum`, `interface`, `impl`, assignment, `<T>` parameter lists on those declarations (which conflict with constraint C-1), and the legacy `static` forms that the plan retires. |
| Parser | `src/parser.rs` | Partial. Grouped expressions become `null` (PARSE-1). Type syntax becomes the identifier `_` (PARSE-2). `struct`, `enum`, `interface`, and `impl` panic at `todo!`, and `x = 5` parses as an operation rather than an assignment (PARSE-3). `for` loses its header and body (PARSE-4). `case` yields no branches (PARSE-5). Annotations and comments are dropped (PARSE-8). |
| AST | `src/ast/` | A plain owned tree of node structs for statements, expressions, patterns, and literals. It is not stored in `hecs`; a per-node `hecs` side table holds only `Position` (AST-9). No declaration nodes for `struct`, `enum`, `interface`, or `impl`. See [ast.md](../ast.md). |
| Visitor | `src/ast/visitor.rs` | Preorder walk functions plus component filters. See [visitor-pattern.md](visitor-pattern.md). |
| Printer | `src/ast/printer.rs` | Pretty-prints nodes, but the output is not valid Patina (PRINT-1). |
| Analysis | `src/analysis/` | Standalone data structures: arena, IDs, symbol table, type registry, stages, and diagnostics. Not connected to the parser or CLI, and there is no resolver or expression type checking (ANL-4). |
| CLI | `src/main.rs` | `patina <input-file>` parses one file and prints counts. |

## Planned Pipeline

The [comptime-reflection plan](../../plans/comptime-reflection/index.md) defines the current direction: incremental per-item queries, inferred comptime/runtime modes, static reflection, and reification. The compiler IR must be a Regionalized Value State Dependence Graph (RVSDG), represented with `hecs` entities and components (constraint C-8). It will be the compiler's first real `hecs` structure, because the AST is not one. It is not implemented yet.

[Step 1](../../plans/comptime-reflection/step-1.md) will design and review the pipeline before implementation planning begins. The former phase-separation roadmap has been removed; its unfinished work remains in the [carry-over table](../../plans/comptime-reflection/index.md#phase-separation-carry-over).

## Parser

- `PatinaParser` derives `pest_derive::Parser` from `src/grammar.pest`.
- `parse_file(input)` calls `parse_file_with_name("", input)`. Both return `Result<Ast, Box<dyn std::error::Error>>`. On failure the boxed error is Pest's own error. The `ParseError` and `ParserContext` types in `parser.rs` are defined but unused.
- `ParserVisitor::parse_root` walks the top-level `decorated_statement` pairs. Each node type implements `ParseNode::parse(ast: &mut Ast, pair: Pair<Rule>) -> Self`, creates an entity, and attaches a `Position`.
- Only the root entity's `Position` records the filename. Every other node gets `filename: None`.
- Defensive `todo!` arms remain in expression, atom, and atomic-value dispatch (PARSE-7). The parser often invents placeholder values instead of returning an error (PARSE-9).

## AST

The AST is a plain owned Rust tree. `Ast::root` is a `Root` whose `statements` hold `Statement` enums, and each node struct stores its children inline (`Vec`, `Box`, `Option`). All traversal and printing work on this tree.

`Ast` also owns a `hecs::World`, and each node carries an `entity: Entity`, but the tree is not stored in the world. The world is a side table keyed by those entities, and the parser puts only a `Position` in it. Outside tests, only the CLI's entity count and the unused component-filter helpers read it. The plan treats the side table as vestigial (AST-9); see [ast.md](../ast.md#the-hecs-side-table).

A `Position` records the start line and column, a byte offset, and a byte length. It has no end line or column (POS-6).

Expressions represent types as well; there is no separate type-expression node. `Operation` is a flat, source-ordered list of operands and operators, so precedence is never resolved (AST-3). Entity IDs are not stable across parses (AST-7).

`Ast<C = ()>` has a Rust type parameter that is used only as a marker; only `Ast<()>` can be constructed. It is unrelated to Patina language generics, which the plan forbids (constraint C-1).

## Analysis (not integrated)

`src/analysis/` holds:

- `Arena<T>`, an append-only `Vec<T>` that returns `Id<T>` indices. `SymbolTable` and `TypeRegistry` store `Id<Symbol>`, `Id<Scope>`, and `Id<Type>`. The separate tag aliases `SymbolId`, `TypeId`, and `ScopeId` are a second, incompatible ID family (ANL-1). The plan replaces both (ANL-9).
- `SymbolTable`, with scopes and stage-aware lookup.
- `TypeRegistry`, `Type`, and `TypeChecker`. `TypeChecker` checks only explicitly supplied code-type and stage relationships (ANL-4). `Type::TypeParameter` and `Type::Generic` conflict with constraint C-1 (ANL-6).
- `Stage(pub u32)`, where stage 0 is compile time and stage 1 is runtime. The plan replaces numbered stages with inferred comptime/runtime modes (constraint C-2), and splits the remaining meaning into item sealing and a reification order under a new name (ANL-10).
- `ErrorCollector` and `Diagnostic`, a mutable diagnostics accumulator (ANL-8).
- `AnalysisContext`, which wraps the above in `Arc<Mutex<_>>` (ANL-5).

## Pointer Syntax

The grammar accepts the pointer types `*own T`, `*mut T`, `*raw T`, and `*T`. The parser discards them as the identifier `_` (PARSE-2), and no ownership semantics are implemented.

## Module Organization

```text
src/
├── main.rs              # CLI: parse one file, print counts
├── lib.rs               # Library root; re-exports ast::* and parser::*
├── grammar.pest         # Pest grammar
├── parser.rs            # PatinaParser, ParserVisitor, ParseNode impls
├── parser/
│   └── tests.rs         # RON fixture tests (test_cases/parser/)
├── test_shared.rs       # RON directory test runners (cfg(test))
├── ast/
│   ├── mod.rs           # Ast<C>, AstNode, GenericVisitor, Traversable
│   ├── nodes.rs         # Node structs and enums
│   ├── components.rs    # Side-table components (Position, ...) and ComponentBox
│   ├── visitor.rs       # Visitor trait, walk_* functions, filters
│   ├── printer.rs       # PrinterVisitor source printer
│   ├── test_utils.rs    # Printer fixture helpers (cfg(test))
│   └── *_tests.rs       # Unit tests attached with #[path]
└── analysis/            # Not connected to the parser
    ├── mod.rs           # Re-exports
    ├── arena.rs         # Arena<T>
    ├── id.rs            # Id<T> and tag aliases
    ├── context.rs       # AnalysisContext
    ├── diagnostics.rs   # ErrorCollector, Diagnostic
    ├── stage.rs         # Stage
    ├── symbol_table.rs  # SymbolTable, Scope, Symbol
    └── types.rs         # TypeRegistry, Type, TypeChecker
```

## Extending the Compiler Today

- **New syntax:** update `src/grammar.pest`, add a node struct with an `entity: Entity` field in `src/ast/nodes.rs` (plus a variant on the enum that contains it), implement `ParseNode` or a parse helper in `src/parser.rs`, add `visit_*`/`walk_*` support in `src/ast/visitor.rs` and a `print_*` method in `src/ast/printer.rs`, and add RON fixtures. See [testing.md](../testing.md).
- **New AST data:** add a field to the node struct in `src/ast/nodes.rs`. The `hecs` side table can technically hold more components (see [ast.md](../ast.md)), but the plan expects to remove it (AST-9), so don't build on it.

Larger changes, including analysis passes and any IR work, should wait for the Step 1 designs.
