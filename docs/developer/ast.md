# AST

The AST in `src/ast/` is designed to be a **plain owned Rust tree**. Node types are ordinary structs and enums (`src/ast/nodes.rs`), and every parent owns its children inline through `Vec`, `Box`, and `Option`. Traversal, printing, and structural questions go through this tree. **The AST must not use `hecs` at all**: no ECS world, node entities, side tables, or ECS components for AST metadata.

The current source has not completed that removal: `Ast` still contains a `hecs::World`, and nodes still carry entity handles used by a position side table. These are legacy implementation details to remove, not part of the target AST architecture. The compiler plan requires `hecs` for the RVSDG IR and semantic analysis. Semantic analyses may store each analysis result as a separate component on the relevant analysis entity; those entities and components are not part of the AST.

## `Ast`

```rust
pub struct Ast {
    pub root: Root,
}
```

- This shows the target shape, not the current struct definition. `root` is the tree: `Root { statements: Vec<Statement> }` owns every top-level statement, and each statement owns its children inline.
- Source-file identity and ranges must remain available outside ECS so diagnostics can resolve and print filenames. The design may keep a source-file table or source map; it must not drop filenames to remove the AST's ECS coupling.
- `Ast::new()`/`Default` should construct an empty owned tree without creating a world or entity.
- The unused Rust `C` marker type parameter is implementation scaffolding, not a Patina language generic (constraint C-1); it should be removed with the AST ECS API.

## Nodes

In the current source, node structs still have public `entity: Entity` fields used as keys for the legacy position side table. The target AST removes these handles and the `AstNode::entity_id` API; node identity for incremental queries must be provided separately from AST allocation and from `hecs` entity allocation.

| Kind | Variants |
|---|---|
| `Statement` | `Module`, `Function`, `Let`, `Return`, `Expression`, `While`, `For`, `Break`, `Block`, `Static` |
| `Expression` | `If`, `Case`, `Lambda`, `Operation`, `Call`, `Index`, `Field`, `Literal`, `Identifier`, `Block`, `Grouped` |
| `Literal` | `Integer` (`i64`), `Float`, `String` (text and interpolation fragments), `Boolean`, `Array`, `Null` |
| `Pattern` | `Identifier`, `Tuple`, `Struct` |

Notes:

- Types are ordinary `Expression`s, for example `Function::return_type: Option<Expression>`.
- `Operation` is a flat list of `OperationElement::{Operand(Expression), Operator(String)}`. Precedence is not resolved (AST-3).
- `Statement::Static`, `Static`, and the `is_static` fields on `Function` and `Let` model the legacy `static` keyword. The plan removes them in favor of inferred comptime/runtime modes (AST-2, constraint C-2).
- There are no nodes for `struct`, `enum`, `interface`, `impl`, or assignment.

## Legacy ECS storage to remove

The current parser creates one entity per node and attaches a `Position` component. The tree's parent/child relationships still live only in the node structs; the world is not the AST representation. Remove `Ast::world`, `component_entities`, the per-node entity fields, `AstNode::entity_id`, and the world/query/filter helpers. Do not migrate any AST metadata or analysis facts into an AST-owned ECS.

`hecs` use outside the AST is planned for the RVSDG and semantic analysis. Analysis results can be separate components attached to the relevant analysis/IR entity, allowing each analysis to contribute its own fact without changing AST nodes. Stable compiler/query identities and source-file identities must remain distinct from allocation-local ECS entities.

### Legacy component declarations (not part of the target AST)

`src/ast/components.rs` currently defines these types for the legacy ECS API. They are not components in the target AST architecture; remove or relocate them as the AST ECS API is removed. Only `Position` is ever attached outside tests.

| Component | Fields | Attached by |
|---|---|---|
| `Position` | `filename: Option<String>`, `line`, `column`, `offset`, `length`. `Default` is line 1, column 1. | Currently attached by the parser to most AST entities. Target source metadata is ordinary owned data/source-map data, not an ECS component. Preserve source-file identity and filename lookup for diagnostics. |
| `Stage` | `stage_number: u32` | Nothing |
| `TypeInfo` | `type_name: String`, `is_resolved: bool` | Nothing |
| `SymbolInfo` | `name: String`, `scope_id: usize`, `is_defined: bool` | Nothing |
| `Comment` | `text: String`, `is_doc_comment: bool` | Nothing |
| `Annotation` | `name: String`, `arguments: Vec<String>` | Nothing |
| `Error` | `message: String`, `error_type: ErrorType` (`ParseError`, `TypeError`, `NameError`, `StageError`) | Nothing |

`ComponentBox` wraps a `Box<dyn Any>` with `new`, `downcast`, and `downcast_mut`. The target AST uses none of these component-storage APIs.

The current ECS `Stage` component duplicates `analysis::Stage` (AST-6). Both model numbered stages that the plan replaces (ANL-10). The target removes this component from the AST; semantic-analysis state belongs to semantic-analysis entities.

## Examples

These snippets show ordinary tree use; they do not depend on ECS storage.

Walk the tree (the normal way to inspect an AST):

```rust
let ast = parse_file("let x = 42")?;
if let Statement::Let(let_stmt) = &ast.root.statements[0] {
    assert!(!let_stmt.is_mutable);
}
```

Source ranges and source-file identity are required for diagnostics, including resolving a filename for user-facing output. Their target API is part of the compiler/source-map design; do not read positions from an AST ECS side table or attach semantic facts to AST nodes.

Print the tree:

```rust
let ast = parse_file("let x = 42\nlet mut y = \"hello\"")?;
let mut printer = PrinterVisitor::new();
printer.print_root(&ast.root);
println!("{}", printer.get_output());
// let x = 42;
// let mut y = "hello";
```

`PrinterVisitor` prints by calling its own `print_*` methods; its `Visitor` impl is empty. The output is not valid Patina: note the trailing semicolons (PRINT-1).

For traversal, see [visitor-pattern.md](architecture/visitor-pattern.md).

## Integration Status

Known gaps, tracked in [findings.md](../plans/comptime-reflection/findings.md):

- **Legacy ECS storage:** the `hecs` world, `component_entities`, the `C` marker, per-node entity fields, and ECS helpers still exist in the source but must be removed from the AST. This is tracked under C-8 and AST-7, not as an accepted AST design.
- **Positions:** a `Position` records only the start line and column, plus a byte length; there is no end line or column (POS-6). The root `Position` records the filename passed to `parse_file_with_name`. Every other `Position` has `filename: None`, and some synthetic placeholder entities have no `Position` at all.
- **Annotations and comments:** parsed, then discarded. No `Annotation` or `Comment` components are attached (PARSE-8). `Annotation::arguments` is `Vec<String>`, not expressions with spans (AST-1).
- **Analysis facts:** `TypeInfo`, `SymbolInfo`, `Stage`, and `Error` are never attached, because `src/analysis/` is not connected to the parser.
- **Stability:** entity IDs depend on allocation order and are not stable across parses (AST-7), so they cannot key incremental caches.
- **Generic traversal:** `GenericVisitor`/`Traversable` have no node implementations (VIS-2).

The plan's Step 1 `ast-and-ir.md` design decides the AST's ordinary Rust storage and stable item identity. The target architecture uses `hecs` for the RVSDG IR and semantic analysis, never for the AST (constraint C-8).
