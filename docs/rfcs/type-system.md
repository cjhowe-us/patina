# Type System Extensions

## Current State

Basic type representation exists but lacks:
- Full type inference
- Generic types
- Type aliases
- Traits/interfaces

## Planned Extensions

### 1. Generic Types

**Parametric Polymorphism**
```patina
// Generic function
fn identity<T>(x: T) -> T {
    x
}

// Generic struct
struct Box<T> {
    value: T
}

// Generic with constraints
fn sort<T: Ord>(items: &mut [T]) {
    // ...
}
```

**Implementation Requirements**:
- Type parameters in AST
- Type substitution
- Constraint solving
- Monomorphization

### 2. Type Aliases

**User-Defined Type Names**
```patina
type NodeId = u64
type Result<T> = Either<T, Error>
type Handler = fn(Event) -> Response
```

**Benefits**:
- Improved readability
- Abstraction over complex types
- Documentation

### 3. Traits/Interfaces

**Abstract Over Types**
```patina
trait Display {
    fn display(self: &Self) -> String
}

trait Iterator<T> {
    fn next(self: &mut Self) -> Option<T>
}

impl Display for i32 {
    fn display(self: &Self) -> String {
        self.to_string()
    }
}
```

**Features Needed**:
- Trait definitions
- Implementation blocks
- Trait bounds
- Default implementations

### 4. Associated Types

**Types Within Traits**
```patina
trait Container {
    type Item
    
    fn get(self: &Self, index: usize) -> Option<&Self::Item>
}

impl Container for Array<T> {
    type Item = T
    
    fn get(self: &Self, index: usize) -> Option<&T> {
        // ...
    }
}
```

## Type Inference Enhancements

### 1. Bidirectional Type Checking

**Inference Modes**:
- **Synthesis**: Infer type from expression
- **Checking**: Check expression against expected type

```rust
enum InferMode {
    Synth,
    Check(TypeId),
}
```

### 2. Constraint Collection

**Type Constraints**:
```rust
enum Constraint {
    Equal(TypeId, TypeId),
    Subtype(TypeId, TypeId),
    HasField(TypeId, String, TypeId),
    Callable(TypeId, Vec<TypeId>, TypeId),
}
```

### 3. Unification Algorithm

**Solve Constraints**:
1. Collect constraints during traversal
2. Build substitution map
3. Apply substitutions
4. Check satisfiability

## Advanced Type Features

### 1. Higher-Kinded Types
```patina
type Functor<F<_>> = trait {
    fn map<A, B>(fa: F<A>, f: fn(A) -> B) -> F<B>
}
```

### 2. Dependent Types (Future)
```patina
fn create_array<n: usize>(default: T) -> [T; n] {
    [default; n]
}
```

### 3. Effect Types
```patina
fn read_file(path: String) -> Result<String> @[effect(IO)]
```

### 4. Linear Types
```patina
struct UniquePtr<T> @[linear] {
    ptr: *own T
}
```

## Implementation Roadmap

### Phase 1: Type Aliases
- Parse type alias syntax
- Store in symbol table
- Resolve during type checking
- Test with complex types

### Phase 2: Basic Generics
- Type parameters on functions
- Type parameter substitution
- Simple constraint solving
- Monomorphization

### Phase 3: Traits
- Trait definitions
- Implementation blocks
- Method resolution
- Trait bounds

### Phase 4: Advanced Features
- Associated types
- Higher-ranked trait bounds
- Type-level computation
- Const generics

## Type System Architecture

### 1. Type Representation
```rust
pub enum Type {
    // Basic types
    Int(IntType),
    Bool,
    String,
    
    // Compound types
    Array(Box<Type>, Option<usize>),
    Pointer(PointerKind, Box<Type>),
    Function(Vec<Type>, Box<Type>),
    
    // Advanced types
    Generic(TypeParam),
    Applied(TypeConstructor, Vec<Type>),
    Trait(TraitId),
    Associated(Type, String),
}
```

### 2. Type Context
```rust
pub struct TypeContext {
    types: Arena<Type>,
    constraints: Vec<Constraint>,
    substitutions: HashMap<TypeParam, TypeId>,
    trait_impls: HashMap<(TraitId, TypeId), ImplId>,
}
```

### 3. Type Checking Pipeline
1. Collect type annotations
2. Infer types for expressions
3. Generate constraints
4. Solve constraints
5. Apply substitutions
6. Verify trait bounds

## Benefits

1. **Expressiveness**: Write more generic, reusable code
2. **Safety**: Catch more errors at compile time
3. **Performance**: Zero-cost abstractions
4. **Ergonomics**: Better type inference reduces annotations
5. **Modularity**: Traits enable better code organization