# Step 1: Foundation and Stage Infrastructure

## Objective

Establish the core infrastructure for stage-based compilation, including data structures, traits, and basic stage representation.

## Tasks

### 1.1 Define Stage Level Types

Create fundamental types for representing stages:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StageLevel {
    Stage0,     // Compile-time
    Stage1,     // Runtime
    StageN(u32) // Higher stages
}
```

### 1.2 Create Stage Context

Implement context for tracking stage information:
- Current stage level during analysis
- Stage constraints and requirements
- Cross-stage reference tracking
- Stage-specific symbol tables

### 1.3 Define Stage Annotations

Design AST annotations for stage information:
- Explicit stage annotations (`@[stage(0)]`)
- Inferred stage markers
- Quote/splice markers
- Cross-stage persistence flags

### 1.4 Establish Stage Interfaces

Create traits for stage-aware components:
- `StageAware` trait for AST nodes
- `StageAnalyzer` trait for analysis passes
- `StageSeparable` trait for AST splitting
- Error types for stage violations

## Acceptance Criteria

- [ ] Stage level types are defined and documented
- [ ] Stage context can track nested stage levels
- [ ] AST nodes can carry stage annotations
- [ ] Stage interfaces are implemented
- [ ] Basic tests for stage representation

## Dependencies

- Existing AST structure
- Type system foundations
