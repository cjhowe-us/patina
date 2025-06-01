# Component-Based AST System

## Overview

Patina uses an Entity-Component-System (ECS) architecture for its AST, powered by the `hecs` crate. This provides compile-time type safety through generics without runtime boxing overhead.

## Architecture

### Entity-Component-System Pattern

The system uses `hecs` ECS:

- **Entities**: AST nodes have an `entity: Entity` field from `hecs`
- **Components**: Type-safe data attached to entities (Position, Stage, TypeInfo, etc.)
- **World**: The `hecs::World` stores all entities and components
- **Generic AST**: `Ast<C>` allows different component types at compile time

### Core AST Structure

```rust
pub struct Ast<C = ()> {
    pub world: World,
    pub statements: Vec<Statement>,
    pub component_entities: HashSet<Entity>,
    pub _phantom: PhantomData<C>,
}
```

The AST is generic over component type `C`, enabling type-safe component operations without boxing.

## Built-in Components

Located in `src/ast/components.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stage {
    pub stage_number: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolInfo {
    pub symbol_id: String,
    pub scope_id: String,
}
```

## AST Nodes

All AST nodes implement the `AstNode` trait:

```rust
pub trait AstNode {
    fn entity(&self) -> Entity;
}
```

Each node contains:
- Its own data (name, parameters, etc.)
- An `entity: Entity` field for ECS integration
- References to child nodes (stored in the parent struct)

## Component Operations

### Adding Components

```rust
// Add a component without boxing
ast.add_component(entity, Stage { stage_number: 0 });
ast.add_component(entity, TypeInfo { type_name: "i32".to_string() });
```

### Querying Components

```rust
// Query entities with specific components
for (entity, stage) in ast.query_entities::<&Stage>().iter() {
    println!("Entity {:?} is at stage {}", entity, stage.stage_number);
}

// Filter entities by component presence
let staged_entities = ast.filter_entities::<Stage>();
```

### Generic Operations

```rust
// Works with any component type
fn process_typed_nodes<C>(ast: &Ast<C>) 
where 
    C: Component + PartialEq
{
    for entity in ast.filter_entities::<TypeInfo>() {
        // Process entities with TypeInfo
    }
}
```

## Visitor Patterns

### Traditional Visitor

The traditional visitor pattern is preserved:

```rust
pub trait Visitor {
    fn visit_statement(&mut self, _stmt: &Statement) {}
    fn visit_expression(&mut self, _expr: &Expression) {}
    // ... methods for each node type
}
```

### Generic Visitor

New generic visitor for component-based traversal:

```rust
pub trait GenericVisitor<C> {
    fn visit_node<N: AstNode>(&mut self, ast: &Ast<C>, node: &N);
}
```

### Component Filter Visitor

Wraps any visitor to filter by components:

```rust
let mut visitor = MyVisitor::new();
let mut filter = ComponentFilterVisitor::<TypeInfo, _>::new(visitor);
ast.walk_with_generic_visitor(&mut filter);
```

## Usage Examples

### Creating an AST with Components

```rust
let mut ast: Ast<Stage> = Ast::new();

// Parse and create nodes
let stmt = parse_statement(input)?;
let entity = stmt.entity();

// Add components
ast.add_component(entity, Stage { stage_number: 0 });
ast.add_component(entity, Position { line: 1, column: 1, offset: 0 });

ast.statements.push(stmt);
```

### Walking the AST

```rust
// Traditional visitor
let mut visitor = PrintVisitor::new();
ast.walk(&mut visitor);

// Generic visitor for component operations
struct StageVisitor;
impl<C> GenericVisitor<C> for StageVisitor {
    fn visit_node<N: AstNode>(&mut self, ast: &Ast<C>, node: &N) {
        if let Ok(stage) = ast.world.get::<&Stage>(node.entity()) {
            println!("Node at stage {}", stage.stage_number);
        }
    }
}
```

### Type-Safe Component Queries

```rust
// Query multiple components at once
for (entity, (stage, type_info)) in ast.query_entities::<(&Stage, &TypeInfo)>().iter() {
    println!("Entity {:?}: stage {}, type {}", 
             entity, stage.stage_number, type_info.type_name);
}
```

## Benefits

1. **Type Safety**: Compile-time checking via generics
2. **Performance**: No boxing overhead, direct component access
3. **Flexibility**: Any type can be a component
4. **Extensibility**: Add new components without modifying nodes
5. **Compatibility**: Traditional visitor pattern still works

## Integration Status

- ✅ Parser creates AST with ECS entities
- ✅ All node types support components
- ✅ Visitor patterns implemented
- ❌ Semantic analysis not yet using components
- ❌ Component data not yet populated during analysis

## Future Work

1. **Populate Components**: Connect semantic analysis to populate TypeInfo, SymbolInfo
2. **Analysis Visitors**: Implement visitors that add analysis components
3. **Optimization**: Use component queries for optimization passes
4. **Serialization**: Save/load AST with components
5. **Parallel Analysis**: Leverage ECS for parallel processing