# Step 5: Type System Integration

## Objective

Extend the type system to be stage-aware, ensuring type safety across stage boundaries.

## Tasks

### 5.1 Stage-Aware Types

Enhance type representation:
- Add stage level to type information
- Track stage constraints on types
- Handle polymorphic stage levels
- Implement stage subtyping rules

### 5.2 Type Checking Across Stages

Implement cross-stage type checking:
- Verify quote expression types
- Check splice type compatibility
- Validate CSP type preservation
- Ensure stage-polymorphic functions

### 5.3 Stage-Dependent Types

Support types that vary by stage:
- Code types for quoted expressions
- Stage-specific type aliases
- Phantom stage parameters
- Dependent stage types (future)

### 5.4 Type Inference with Stages

Extend type inference:
- Infer stage levels with types
- Resolve stage constraints
- Handle stage polymorphism
- Generate helpful error messages

## Acceptance Criteria

- [ ] Types correctly track stage information
- [ ] Cross-stage type errors are caught
- [ ] Type inference handles stages
- [ ] Stage polymorphism works correctly
- [ ] Error messages mention stage issues

## Dependencies

- Basic type system implementation
- Stage infrastructure from Step 1
