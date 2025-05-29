# Stage-Based Metaprogramming in Patina

Patina implements a stage-based metaprogramming system inspired by MacroCaml, enabling powerful compile-time computation while maintaining type safety and clear separation between compilation phases.

## Core Concepts

### Stage Levels

Patina code executes at different stages:

- **Stage level 0**: Runtime stage. Never executed at compile time.
- **Stage level 1**: Compile-time stage 1. Executed first
- **Stage level n** (n > 1): Compile-time stages in increasing order

### Stage Annotations

Stage levels are specified using static declarations or expressions:

```patina
// Stage level 1: Variables that do not depend on runtime values are compile-time values
let constant_value = 100
let mut mutable_value = 200
static let compile_time_constant = 42

// Stage level 2: Compile-time constant that depends on a stage 1 constant
static let generated = compute_at_compile_time(compile_time_constant)

// Non-static functions are polymorphic over the stage level
fun nonStaticFun(): i32 {
    // If the caller is static, this will be a compile-time value in stage level 1, otherwise 0 (runtime).
    let mut mutable_runtime_value = 1234
    // If the caller is static, this will be stage level 2, otherwise 0 (runtime).
    let implicit_runtime_value = mutable_runtime_value + 100
    // Return is stage level 2. So this will be the stage of the function call
    return implicit_runtime_value
}
// Stage level 3: Compile-time constant that calls a stage level 2 function to initialize
let x = static nonStaticFun()

// Fails compilation if it references a runtime value
static let runtimeValue =
```

## Design Principles

### 1. Explicit Staging
All stage transitions must be explicit, preventing accidental mixing of compile-time and runtime values. This is achieved through the `static` keyword.

### 2. Type Safety
The type system tracks stage levels, ensuring:
- No runtime-only values in compile-time expressions
- No unquoted compile-time values at runtime
- Proper escaping of generated code
- No circular dependencies between stage levels

## Implementation

### AST Representation

Stage information is stored in AST nodes:
```rust
pub struct StageInfo {
    pub level: StageLevel,
    pub source: StageSource,
}

pub enum StageLevel {
    Known(usize),
    Inferred,
}
```

### Stage Analysis

The semantic analyzer (`src/analysis/stage.rs`) performs:

1. **Stage Inference**: Deduce stages for unannotated code
2. **Stage Validation**: Ensure stage consistency
3. **Cross-Stage Checking**: Validate stage transitions

### Symbol Table Integration

Symbols track their stage level:
```rust
pub struct Symbol {
    pub name: String,
    pub stage: StageLevel,
    pub type_info: TypeInfo,
    // ...
}
```

## Usage Examples

### Compile-Time Constants
```patina
@[stage(0)]
let TABLE_SIZE = 1024

@[stage(0)]
let MAGIC_NUMBER = compute_magic_number()

// Used at runtime
let table: [u8; $(TABLE_SIZE)] = init_table()
```

### Code Generation
```patina
@[stage(0)]
fn generate_accessor(field: string) -> Expr {
    // Generate getter function
    quote {
        fn $("get_" + field)(self: &Self) -> &FieldType {
            &self.$(field)
        }
    }
}

// Generate at compile time
@[stage(0)]
let getter = generate_accessor("value")

// Available at runtime
$(getter)
```

### Staged Computation
```patina
// Stage 0: Generate specialized function
@[stage(0)]
fn specialize_sort(type: Type) -> Function {
    match type {
        Type::Int => generate_int_sort(),
        Type::Float => generate_float_sort(),
        _ => generate_generic_sort(type)
    }
}

// Stage 1: Use specialized function
let sorted = $(specialize_sort(Int))(my_array)
```

## Stage Transitions

### Quote (`quote { ... }`)
Delays computation to the next stage:
```patina
@[stage(0)]
let code = quote { x + y }  // Not evaluated yet
```

### Splice (`$(...)`)
Evaluates expression at current stage and inserts result:
```patina
@[stage(0)]
let value = 42

@[stage(1)]
let result = $(value) + 10  // 52 at runtime
```

### Lift (`lift(...)`)
Explicitly lifts values across stages:
```patina
@[stage(0)]
let config = load_config()

@[stage(1)]
let runtime_config = lift(config)
```

## Restrictions

### Stage Consistency
- Lower stage code cannot reference higher stage values
- Functions must have consistent stage signatures
- Type parameters can have stage constraints

### Memory Safety
- Pointers cannot cross stage boundaries
- References are valid only within their stage
- Ownership rules apply within each stage

## Benefits

1. **Performance**: Expensive computations at compile time
2. **Specialization**: Generate optimized code for specific types
3. **Configuration**: Embed configuration without runtime overhead
4. **Metaprogramming**: Build DSLs and code generators
5. **Type Safety**: Stage checking prevents runtime errors

## Future Extensions

- **Stage Polymorphism**: Functions generic over stage levels
- **Dependent Stages**: Stage levels computed from values
- **Multi-Stage Debugging**: Debug across compilation stages
- **Stage Optimization**: Automatic stage inference and lifting
