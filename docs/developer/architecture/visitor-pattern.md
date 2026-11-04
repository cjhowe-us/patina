# Visitor Pattern

AST traversal lives in `src/ast/visitor.rs`, which `patina::ast` and the crate root re-export. It has three parts:

1. The `Visitor` trait, with one no-op default method per node kind.
2. Free `walk_*` functions that recurse through the node tree and call the visitor.
3. Filtering helpers over the AST's `hecs` side table, which holds only `Position` today ([ast.md](../ast.md#the-hecs-side-table)): `ComponentFilterVisitor`, `FilterTraverser`, and `ComponentFilterWrapper`. None has a non-test caller.

A separate `GenericVisitor`/`Traversable` pair in `src/ast/mod.rs` exists but is not used by any node (VIS-2).

## `Visitor`

Every method takes `(&mut self, ast: &Ast, node: &NodeType)` and defaults to doing nothing, so a visitor overrides only what it needs:

| Group | Methods |
|---|---|
| Statements | `visit_module`, `visit_function`, `visit_let`, `visit_return`, `visit_while`, `visit_for`, `visit_break`, `visit_block`, `visit_static` |
| Expressions | `visit_expression` (called for every expression), `visit_identifier`, `visit_literal`, `visit_operation`, `visit_if`, `visit_case`, `visit_lambda`, `visit_call`, `visit_index`, `visit_field` |
| Literals | `visit_integer_literal`, `visit_float_literal`, `visit_string_literal`, `visit_boolean_literal`, `visit_array_literal`, `visit_null_literal` |
| Patterns | `visit_tuple_pattern`, `visit_struct_pattern` (identifier patterns call `visit_identifier`) |
| Other | `visit_parameter`, `visit_if_branch`, `visit_case_branch` |

There is no `visit_statement` and no `visit_root`. Expression statements and block statements produce no statement-level callback; the walk goes straight to `walk_expression` or `walk_block`. `visit_static` covers the legacy `static { ... }` block, which the [comptime-reflection plan](../../plans/comptime-reflection/index.md) retires (constraint C-2).

## Walk Functions

| Function | Behavior |
|---|---|
| `walk_ast(visitor, ast)` | Calls `walk_statement` on each `ast.root.statements` entry. |
| `walk_statement(visitor, ast, stmt)` | Calls the statement's `visit_*` method, then walks its children. |
| `walk_expression(visitor, ast, expr)` | Calls `visit_expression`, then the specific method, then recurses. `Grouped` and `Block` expressions recurse without a specific callback. |
| `walk_block(visitor, ast, block)` | Calls `visit_block`, then walks each statement. |
| `walk_pattern(visitor, ast, pattern)` | Visits identifier, tuple, and struct patterns, then their sub-patterns. |
| `walk_literal`, `walk_string_literal`, `walk_array_literal` | Dispatch to the literal callbacks, then walk interpolations or array elements. |
| `traverse_generic(visitor, ast, node)` | Calls `node.accept(visitor, ast)` for the `GenericVisitor` API below. |

Traversal is preorder: a parent's callback runs before its children's. Child order is:

- `Function`: each parameter (`visit_parameter`, then its type expression), then the return type, then the body.
- `Let`: the pattern, then the value, then the type annotation.
- `For`: init, condition, update, then body.
- `Case`: the scrutinee, then for each branch its pattern, body, and guard.

For example, walking `let x = 1 + y` calls, in order: `visit_let`, `visit_identifier` (pattern `x`), `visit_expression` + `visit_operation`, `visit_expression` + `visit_literal` + `visit_integer_literal`, `visit_expression` + `visit_identifier` (`y`).

```rust
use patina::*;

struct IdentCounter {
    count: usize,
}

impl Visitor for IdentCounter {
    fn visit_identifier(&mut self, _ast: &Ast, _ident: &Identifier) {
        self.count += 1;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ast = parse_file("let x = a + b")?;
    let mut counter = IdentCounter { count: 0 };
    walk_ast(&mut counter, &ast);
    assert_eq!(counter.count, 3);
    Ok(())
}
```

## Component Filters

### `ComponentFilterVisitor<T>`

Records entities that carry a `T` component. It does not implement `Visitor`. Call `visit_if_has_component(ast, node)` on any `AstNode`, then read the public `visited_entities: HashSet<Entity>`.

### `FilterTraverser<T>`

Iterates the side-table world directly rather than the node tree, so it never sees tree structure:

```rust
let mut count = 0;
FilterTraverser::<Position>::new().traverse(&ast, |_entity, pos| {
    count += 1;
    let _ = (pos.line, pos.column);
});
```

`traverse_mut(&mut ast, |entity, &mut T| ...)` is the mutable form. Iteration order follows the `hecs` archetype layout, not source order.

### `ComponentFilterWrapper<V, F>`

Wraps an inner visitor and forwards a callback only when `filter(ast, node_entity)` returns `true`. `F: Fn(&Ast, Entity) -> bool`. The `inner` and `filter` fields are public.

```rust
let mut wrapped = ComponentFilterWrapper::new(
    IdentCounter { count: 0 },
    |ast: &Ast, entity| ast.get_component::<Position>(entity).is_some(),
);
walk_ast(&mut wrapped, &ast);
let count = wrapped.inner.count;
```

The wrapper does not forward `visit_expression` or `visit_literal`, so an inner visitor never receives those callbacks (VIS-1).

## `GenericVisitor` and `Traversable`

`src/ast/mod.rs` declares:

```rust
pub trait GenericVisitor<C> {
    fn visit_node<N: AstNode>(&mut self, ast: &Ast<C>, node: &N)
    where
        Self: Sized;
}

pub trait Traversable<C> {
    fn accept<V: GenericVisitor<C>>(&self, visitor: &mut V, ast: &Ast<C>)
    where
        Self: AstNode + Sized,
    {
        visitor.visit_node(ast, self);
    }
}
```

No AST node implements `Traversable`, and the default `accept` visits only the node itself, without recursing (VIS-2). `C` here is the Rust marker type of `Ast<C>`; it is not a Patina language generic.

## Limitations

These are tracked in [findings.md](../../plans/comptime-reflection/findings.md):

- **VIS-1:** `ComponentFilterWrapper` drops `visit_expression` and `visit_literal`.
- **VIS-2:** `GenericVisitor`/`Traversable` have no node implementations and do not recurse.
- **VIS-3:** Traversal is preorder only, always recurses, has no enter/exit hooks, and receives only `&Ast`. Scoped name resolution and per-node fact storage cannot be built on it.

The traversal and pass model that replaces this is a Step 1 design task in the [comptime-reflection plan](../../plans/comptime-reflection/step-1.md). If the AST's side table is removed (AST-9), the component filters go with it.
