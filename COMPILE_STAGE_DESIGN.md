# Compile Stage Level Design for Patina

Based on MacroCaml's approach to compile-time staging, this document outlines how Patina should implement compile stage levels instead of traditional phase separation.

## Key Concepts from MacroCaml

### Compile Stage Levels

In MacroCaml, code exists at different compile-time stages:

- **Stage 0**: Compile-time code (macros, type computations)
- **Stage 1**: Runtime code (normal program execution)
- **Stage n**: Code that will be generated at stage n-1

### Core Mechanisms

1. **Quotation**: `<expr>` quotes an expression to delay its evaluation
2. **Splicing**: `~expr` splices a quoted expression into surrounding code
3. **Stage Polymorphism**: Functions can be polymorphic over stages
4. **Type-Level Computation**: Types can be computed at compile time

## Patina's Compile Stage Architecture

### Stage Definitions

```rust
// src/analysis/stage.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stage(pub u32);

impl Stage {
    pub const COMPILE: Stage = Stage(0);
    pub const RUNTIME: Stage = Stage(1);

    pub fn next(self) -> Stage {
        Stage(self.0 + 1)
    }

    pub fn prev(self) -> Option<Stage> {
        if self.0 > 0 {
            Some(Stage(self.0 - 1))
        } else {
            None
        }
    }
}
```

### Staged AST Design

Instead of separate AST types for each phase, we annotate AST nodes with their stage:

```rust
// src/ast/staged.rs
use crate::analysis::Stage;

pub trait Staged {
    fn stage(&self) -> Stage;
}

// Extend existing AST nodes with stage information
pub struct StagedExpression {
    pub expr: Expression,
    pub stage: Stage,
}

pub struct StagedType {
    pub ty: Type,
    pub stage: Stage,
}

// Quotation and splicing nodes
pub enum StagedNode {
    Quote {
        expr: Box<StagedExpression>,
        from_stage: Stage,
        to_stage: Stage,
    },
    Splice {
        expr: Box<StagedExpression>,
        from_stage: Stage,
        to_stage: Stage,
    },
}
```

### Stage-Aware Symbol Table

```rust
// Extend symbol table to track stages
pub struct StagedSymbol {
    pub symbol: Symbol,
    pub defined_at_stage: Stage,
    pub available_at_stages: Vec<Stage>,
}

impl SymbolTable {
    pub fn declare_staged(
        &mut self,
        name: String,
        position: Position,
        stage: Stage,
        kind: SymbolKind,
    ) -> Result<Id<Symbol>, SymbolError> {
        // Check stage constraints
        // Symbols from stage 0 can be used at any stage
        // Symbols from stage n can only be used at stage n or in quotes
        todo!()
    }

    pub fn lookup_at_stage(
        &self,
        name: &str,
        current_stage: Stage,
    ) -> Option<Id<Symbol>> {
        // Find symbol and check stage compatibility
        todo!()
    }
}
```

### Stage-Aware Type System

```rust
// Types can depend on stage
pub enum StagedTypeKind {
    // Regular types exist at all stages
    Regular(Type),

    // Code types represent quoted code
    Code {
        inner_type: Box<StagedType>,
        from_stage: Stage,
    },

    // Stage-polymorphic types
    StagePolymorphic {
        var: StageVar,
        inner: Box<StagedType>,
    },
}
```

### Semantic Analysis with Stages

```rust
pub struct StageAnalyzer {
    current_stage: Stage,
    symbol_table: SymbolTable,
    type_registry: TypeRegistry,
    errors: ErrorCollector,
}

impl StageAnalyzer {
    pub fn analyze_quote(&mut self, expr: &Expression) -> Result<StagedExpression, Error> {
        // Enter next stage
        let prev_stage = self.current_stage;
        self.current_stage = self.current_stage.next();

        // Analyze expression at new stage
        let result = self.analyze_expression(expr)?;

        // Restore stage
        self.current_stage = prev_stage;

        Ok(StagedExpression {
            expr: result,
            stage: self.current_stage.next(),
        })
    }

    pub fn analyze_splice(&mut self, expr: &Expression) -> Result<StagedExpression, Error> {
        // Splices must come from previous stage
        match self.current_stage.prev() {
            Some(prev_stage) => {
                // Check that expr has type Code<T> at previous stage
                todo!()
            }
            None => {
                self.errors.error(
                    "Cannot splice at stage 0".to_string(),
                    expr.position().clone(),
                );
                Err(Error::InvalidStage)
            }
        }
    }
}
```

## Implementation Plan

### Phase 1: Stage Infrastructure

1. Implement `Stage` type and operations
2. Add stage tracking to symbol table
3. Create stage-aware type representations

### Phase 2: AST Extensions

1. Add `Staged` trait to existing AST
2. Implement quotation and splicing nodes
3. Update parser to handle stage syntax

### Phase 3: Stage Analysis

1. Implement `StageAnalyzer`
2. Add stage checking to name resolution
3. Implement stage-aware type checking

### Phase 4: Code Generation

1. Stage 0 evaluation (compile-time computation)
2. Quote/splice elimination
3. Multi-stage code generation

## Benefits Over Traditional Phase Separation

1. **Unified AST**: No need for multiple AST types
2. **Flexible Staging**: Can have arbitrary number of stages
3. **Metaprogramming**: Natural support for macros and code generation
4. **Type Safety**: Stage errors caught at compile time
5. **Incremental**: Can add staging to existing language

## Integration with Current Implementation

The existing symbol table and type system can be extended with stage information rather than replaced:

1. Add `stage: Stage` field to `Symbol`
2. Extend `TypeId` to include stage information
3. Modify lookup functions to be stage-aware
4. Keep existing infrastructure, just make it stage-aware

This approach is more incremental and builds on the solid foundation already implemented.
