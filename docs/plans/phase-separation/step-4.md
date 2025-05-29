# Step 4: Stage 0 Evaluation

## Objective

Implement the compile-time evaluation engine for Stage 0 code that generates Stage 1 AST.

## Tasks

### 4.1 Build Stage0Evaluator

Create the compile-time interpreter:
- Value representation for Stage 0
- Expression evaluation engine
- Built-in function support
- Memory management for compile-time

### 4.2 Code Generation from Stage 0

Implement AST generation:
- Quote expression handling
- AST construction primitives
- Type-safe AST building
- Hygiene for generated names

### 4.3 Cross-Stage Persistence

Handle values crossing stages:
- Serialize Stage 0 values
- Generate Stage 1 literals
- Handle complex data structures
- Maintain type information

### 4.4 Compile-Time Libraries

Provide Stage 0 functionality:
- File I/O for configuration
- String manipulation
- Collection operations
- AST manipulation utilities

## Acceptance Criteria

- [ ] Basic expressions evaluate correctly
- [ ] Quote expressions generate valid AST
- [ ] CSP works for primitive types
- [ ] Compile-time functions execute properly
- [ ] Generated code is hygienic

## Dependencies

- Step 3: Stage separation complete
- Type system for Stage 0
