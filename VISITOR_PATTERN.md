# Visitor Pattern Implementation in Patina

This document explains how the visitor pattern has been implemented in the Patina AST to enable generic tree traversal without match statements.

## Overview

The implementation leverages Rust's trait system to provide a clean interface for AST traversal:

1. **SyntaxNode Trait**: All AST nodes implement this trait, exposing their position and children
2. **Generic Walk Functions**: Multiple walk functions that work with any node implementing SyntaxNode

## Core Components

### SyntaxNode Trait (src/ast/mod.rs)

```rust
pub trait SyntaxNode {
    fn position(&self) -> &Position;
    fn children(&self) -> Vec<&dyn SyntaxNode>;
}
```

This simple trait is implemented by all AST nodes, providing:
- Access to position information
- Access to child nodes as trait objects

### Walk Functions (src/ast/walk.rs)

Several generic walk functions are provided:

- `walk_ast`: Basic traversal with a callback
- `walk_ast_with_callbacks`: Pre- and post-order traversal
- `walk_ast_until`: Early termination support
- `walk_ast_map`: Collect results during traversal


## Benefits

1. **No Match Statements**: The visitor pattern eliminates the need for match statements when traversing the AST
2. **Extensibility**: New visitors can be added without modifying existing code
3. **Type Safety**: Rust's type system ensures correct implementation
4. **Reusability**: Generic walk functions can be reused for different purposes
5. **Separation of Concerns**: Tree structure is separate from traversal logic

## Usage Example

```rust
use patina::ast::{SyntaxNode, walk_ast};

// Count all nodes in an AST
let mut count = 0;
walk_ast(root, &mut |_node| {
    count += 1;
});

// Find nodes at a specific position
let nodes = walk_ast_map(root, &mut |node| {
    if node.position().line == 42 {
        Some(node.position().clone())
    } else {
        None
    }
});
```

## Implementation Notes

- The `dyn SyntaxNode` trait object approach provides maximum flexibility
- Lifetime parameters ensure memory safety when collecting references
- The pattern supports both immutable and mutable traversal (though mutable traversal would require additional trait methods)
- Performance is comparable to match-based traversal due to virtual dispatch optimization in Rust

This implementation demonstrates how trait-based design can create elegant, extensible solutions for complex tree traversal problems.