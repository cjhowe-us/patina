# Visitor Pattern in Patina

The Patina compiler implements a generic visitor pattern for AST traversal, eliminating the need for repetitive match statements while maintaining type safety and extensibility.

## Overview

The visitor pattern leverages Rust's trait system to provide a clean interface for walking the AST:

1. **SyntaxNode Trait**: Common interface for all AST nodes
2. **Generic Walk Functions**: Reusable traversal utilities
3. **Type-Safe Callbacks**: Visitor functions with full type information

## Core Components

### SyntaxNode Trait

All AST nodes implement this trait:

```rust
pub trait SyntaxNode {
    fn position(&self) -> &Position;
    fn children(&self) -> Vec<&dyn SyntaxNode>;
}
```

This provides:
- Position information for error reporting
- Access to child nodes for traversal

### Walk Functions

Located in `src/ast/walk.rs`, these functions provide different traversal strategies:

#### Basic Walk
```rust
pub fn walk_ast<F>(node: &dyn SyntaxNode, callback: &mut F)
where
    F: FnMut(&dyn SyntaxNode),
```
Simple depth-first traversal with a callback for each node.

#### Pre/Post Order Walk
```rust
pub fn walk_ast_with_callbacks<F, G>(
    node: &dyn SyntaxNode,
    pre_callback: &mut F,
    post_callback: &mut G,
)
```
Allows actions before and after visiting children.

#### Early Termination
```rust
pub fn walk_ast_until<F>(node: &dyn SyntaxNode, callback: &mut F) -> bool
where
    F: FnMut(&dyn SyntaxNode) -> bool,
```
Stops traversal when callback returns false.

#### Collecting Results
```rust
pub fn walk_ast_map<F, T>(node: &dyn SyntaxNode, callback: &mut F) -> Vec<T>
where
    F: FnMut(&dyn SyntaxNode) -> Option<T>,
```
Collects values during traversal.

## Usage Examples

### Counting Nodes
```rust
let mut count = 0;
walk_ast(root, &mut |_node| {
    count += 1;
});
println!("Total nodes: {}", count);
```

### Finding Specific Nodes
```rust
let functions = walk_ast_map(root, &mut |node| {
    if let Some(func) = node.downcast_ref::<FunctionStatement>() {
        Some(func.name.clone())
    } else {
        None
    }
});
```

### Validation
```rust
let mut errors = Vec::new();
walk_ast_with_callbacks(
    root,
    &mut |node| {
        // Pre-order: check node validity
        if let Some(error) = validate_node(node) {
            errors.push(error);
        }
    },
    &mut |_node| {
        // Post-order: cleanup if needed
    },
);
```

### Early Exit
```rust
let found = walk_ast_until(root, &mut |node| {
    if node.position().line == target_line {
        println!("Found node at target line");
        false // Stop traversal
    } else {
        true // Continue
    }
});
```

## Benefits

1. **No Match Statements**: Generic traversal without node-specific code
2. **Composability**: Combine different walk strategies
3. **Type Safety**: Rust's type system ensures correctness
4. **Performance**: Zero-cost abstractions, comparable to manual traversal
5. **Maintainability**: New node types automatically work with existing visitors

## Implementation Details

- Uses `dyn SyntaxNode` trait objects for flexibility
- Lifetime parameters ensure memory safety
- Virtual dispatch overhead is minimal in practice
- Stack-based traversal avoids heap allocations

## Advanced Patterns

### Context Threading
```rust
struct Context {
    scope_depth: usize,
    symbols: HashMap<String, Symbol>,
}

let mut ctx = Context::default();
walk_ast(root, &mut |node| {
    // Access and modify context during traversal
    if let Some(block) = node.downcast_ref::<Block>() {
        ctx.scope_depth += 1;
    }
});
```

### Multi-Pass Analysis
```rust
// First pass: collect symbols
let symbols = walk_ast_map(root, &mut |node| {
    extract_symbol(node)
});

// Second pass: resolve references
walk_ast(root, &mut |node| {
    if let Some(ref_node) = node.downcast_ref::<Reference>() {
        resolve_reference(ref_node, &symbols);
    }
});
```

## Future Extensions

The visitor pattern can be extended for:
- Mutable AST transformation
- Parallel traversal
- Incremental recompilation
- Custom traversal orders