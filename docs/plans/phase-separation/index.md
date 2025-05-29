# Stage-Based Phase Separation Plan

## Overview

Patina implements a stage-based compilation model inspired by MacroCaml, where code exists at different compile-time stages. This enables powerful metaprogramming while maintaining type safety across stage boundaries.

## Context

### Stage Levels
- **Stage 0**: Compile-time code (macros, type-level computations, code generation)
- **Stage 1**: Runtime code (normal program execution)  
- **Stage n**: Multi-stage code generation (each stage generates code for stage n+1)

### Motivation
Traditional phase separation treats all code uniformly. Stage-based separation recognizes that some code executes at compile time to generate other code, enabling:
- Zero-cost abstractions through compile-time evaluation
- Type-safe code generation
- Powerful macro systems
- Domain-specific language embedding

## High-Level Plan

### Compilation Pipeline

1. **Parse Phase**
   - Input: Source text
   - Output: Unified AST without stage information
   - Responsibility: Syntax analysis

2. **Stage Analysis Phase**
   - Input: Unified AST
   - Output: AST with stage annotations
   - Responsibility: Calculate stage levels for all expressions

3. **Stage Separation Phase**
   - Input: Stage-annotated AST
   - Output: Stage-specific AST representations
   - Responsibility: Split AST by stage level

4. **Stage 0 Evaluation Phase**
   - Input: Stage 0 AST
   - Output: Generated Stage 1 AST
   - Responsibility: Execute compile-time code

5. **Code Generation Phase**
   - Input: Stage 1 AST
   - Output: Target code
   - Responsibility: Generate executable code

### Key Components

- **StageCalculator**: Determines stage level of expressions
- **StageSeparator**: Splits unified AST into stage-specific ASTs
- **Stage0Evaluator**: Executes compile-time computations
- **Stage-Aware Type System**: Ensures type safety across stages

### Stage Transitions

- **Quote (`quote { ... }`)**: Delays computation to next stage
- **Splice (`${ ... }`)**: Inserts result from previous stage

## Benefits

1. **Compile-Time Computation**: Execute expensive operations at compile time
2. **Code Generation**: Generate specialized code based on compile-time values
3. **Type Safety**: Prevent mixing of stage levels
4. **Zero-Cost Abstractions**: No runtime overhead for compile-time features
5. **Extensibility**: Build powerful macro systems

## Success Criteria

- Clear separation between stage levels
- Type-safe stage transitions
- Efficient compile-time evaluation
- Meaningful error messages for stage violations
- Performance improvements from compile-time computation

## Implementation Steps

See numbered step documents in this directory for detailed implementation plans:
- Foundation and Infrastructure
- Parse and Stage Analysis
- Type System Extensions
- Stage Evaluation
- Integration and Testing
- Documentation and Polish