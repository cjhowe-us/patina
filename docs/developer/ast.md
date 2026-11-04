# AST

The AST in `src/ast/` is a **plain owned Rust tree**. Node types are ordinary structs and enums (`src/ast/nodes.rs`), and every parent owns its children inline through `Vec`, `Box`, and `Option`. Traversal, printing, and every structural question go through this tree.

The AST is **not** an Entity-Component-System structure. `Ast` also holds a `hecs::World`, and each node carries a `hecs::Entity`, but the tree is not stored in the world. The world is a per-node side table, and the parser stores only one thing in it: a `Position` component. Nothing else is attached outside tests, and only a few helpers read the world (see [The `hecs` side table](#the-hecs-side-table)).

Among planned structures, only the compiler IR must use `hecs` (an RVSDG, [constraint C-8](../plans/comptime-reflection/index.md#constraints)). The plan treats the AST's side table as vestigial and expects to replace it with inline spans (AST-9).

## `Ast`

```rust
pub struct Ast<C = ()> {
    pub world: hecs::World,
    pub component_entities: HashSet<Entity>,
    pub root: Root,
    // private PhantomData<C>
}
```

- `root` is the tree. `Root { entity, statements: Vec<Statement> }` owns every top-level statement, and each statement owns its children.
- `world` and `component_entities` are the side table described [below](#the-hecs-side-table). Neither one holds tree structure.
- `Ast::new()` and `Default` exist only for `Ast<()>`. They create an empty world, spawn an entity for the root, and return an empty `Root`.
- `C` is a Rust marker type parameter with no current use. It has nothing to do with Patina language generics, which the [comptime-reflection plan](../plans/comptime-reflection/index.md) forbids (constraint C-1).
- `Debug` prints the world's entity count, not its contents.

## Nodes

Every node struct has a public `entity: Entity` field. Its only purpose is to serve as the key for the node's row in the side table. The `AstNode` trait (`fn entity_id(&self) -> Entity`) is implemented for `Root`, `Statement`, `Parameter`, `Pattern`, `Expression`, `IfBranch`, `CaseBranch`, and `Literal`.

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

## The `hecs` side table

The parser creates one entity per node with `Ast::create_entity()`, which spawns an entity with no components. It stores that entity in the node's `entity` field and then attaches a `Position`. The tree's parent/child relationships live only in the node structs; the world holds no structure.

Methods on `Ast<C>`:

| Method | Behavior |
|---|---|
| `create_entity() -> Entity` | Spawns an empty entity. |
| `add_component<T>(entity, component)` | Inserts or replaces `T` on the entity and records the entity in `component_entities`. Panics if the entity does not exist. |
| `get_component<T>(entity) -> Option<hecs::Ref<T>>` | Shared borrow of a component. |
| `get_component_mut<T>(entity) -> Option<hecs::RefMut<T>>` | Mutable borrow of a component. |
| `query_entities<Q: hecs::Query>() -> hecs::QueryBorrow<Q>` | Raw `hecs` query over the world. |
| `filter_entities<T>() -> HashSet<Entity>` | Every entity that has a `T`. |

Component types must be `'static + Send + Sync`.

Outside tests, the world is read only by:

- the CLI, which prints `ast.world.len()` (`src/main.rs`);
- the filtering helpers in `src/ast/visitor.rs` (`ComponentFilterVisitor` and `FilterTraverser`), which have no non-test callers (see [visitor-pattern.md](architecture/visitor-pattern.md#component-filters)).

The printer, the `Visitor` walk functions, and `src/analysis/` never touch it.

### Components

`src/ast/components.rs` defines these types. All derive `Debug`, `Clone`, `PartialEq`, `Serialize`, and `Deserialize`. Only `Position` is ever attached outside tests.

| Component | Fields | Attached by |
|---|---|---|
| `Position` | `filename: Option<String>`, `line`, `column`, `offset`, `length`. `Default` is line 1, column 1. | Parser, on most entities. Synthetic placeholders, such as the empty `for` body or the `case` scrutinee, get none. |
| `Stage` | `stage_number: u32` | Nothing |
| `TypeInfo` | `type_name: String`, `is_resolved: bool` | Nothing |
| `SymbolInfo` | `name: String`, `scope_id: usize`, `is_defined: bool` | Nothing |
| `Comment` | `text: String`, `is_doc_comment: bool` | Nothing |
| `Annotation` | `name: String`, `arguments: Vec<String>` | Nothing |
| `Error` | `message: String`, `error_type: ErrorType` (`ParseError`, `TypeError`, `NameError`, `StageError`) | Nothing |

`ComponentBox` wraps a `Box<dyn Any>` with `new`, `downcast`, and `downcast_mut`. `Ast` does not use it.

The ECS `Stage` component duplicates `analysis::Stage` (AST-6). Both model the numbered stages that the plan replaces (ANL-10).

## Examples

These snippets compile against the current crate (`use patina::*;`).

Walk the tree (the normal way to inspect an AST):

```rust
let ast = parse_file("let x = 42")?;
if let Statement::Let(let_stmt) = &ast.root.statements[0] {
    assert!(!let_stmt.is_mutable);
}
```

Read a node's span from the side table:

```rust
let ast = parse_file("let x = 42")?;
let entity = ast.root.statements[0].entity_id();
let pos = ast.get_component::<Position>(entity).unwrap();
assert_eq!((pos.line, pos.column), (1, 1));
```

Attach and query an extra component. Nothing outside tests does this:

```rust
let mut ast = parse_file("let x = 42")?;
let entity = ast.root.statements[0].entity_id();

ast.add_component(entity, TypeInfo {
    type_name: "i32".to_string(),
    is_resolved: false,
});

for (entity, (pos, ty)) in ast.query_entities::<(&Position, &TypeInfo)>().iter() {
    println!("{entity:?} at {}:{} has type {}", pos.line, pos.column, ty.type_name);
}

assert_eq!(ast.filter_entities::<TypeInfo>().len(), 1);
```

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

- **Side table:** the `hecs` world, `component_entities`, the `C` marker, and the per-node `entity` fields exist only to hold `Position`. The plan expects to store spans inline and remove them (AST-9).
- **Positions:** a `Position` records only the start line and column, plus a byte length; there is no end line or column (POS-6). The root `Position` records the filename passed to `parse_file_with_name`. Every other `Position` has `filename: None`, and some synthetic placeholder entities have no `Position` at all.
- **Annotations and comments:** parsed, then discarded. No `Annotation` or `Comment` components are attached (PARSE-8). `Annotation::arguments` is `Vec<String>`, not expressions with spans (AST-1).
- **Analysis facts:** `TypeInfo`, `SymbolInfo`, `Stage`, and `Error` are never attached, because `src/analysis/` is not connected to the parser.
- **Stability:** entity IDs depend on allocation order and are not stable across parses (AST-7), so they cannot key incremental caches.
- **Generic traversal:** `GenericVisitor`/`Traversable` have no node implementations (VIS-2).

The plan's Step 1 `ast-and-ir.md` design decides the AST's final storage and item identity. Only the RVSDG IR is required to use `hecs` (constraint C-8).
