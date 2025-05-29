# Step 2: Parse AST and Stage Analysis

## Objective

Implement the stage analysis phase that calculates stage levels for all expressions in the unified AST.

## Tasks

### 2.1 Implement StageCalculator

Create the core stage calculation logic:
- Traverse AST and compute stage levels
- Handle explicit stage annotations
- Infer stages for unannotated code
- Validate stage consistency rules

### 2.2 Stage Inference Rules

Implement inference algorithm:
- Default to Stage 1 (runtime) for unannotated code
- Propagate stages through expressions
- Handle function calls across stages
- Manage stage transitions (quote/splice)

### 2.3 Cross-Stage Validation

Ensure stage safety:
- Prevent higher stage references from lower stages
- Validate quote/splice usage
- Check function stage compatibility
- Enforce cross-stage persistence rules

### 2.4 Stage Analysis Diagnostics

Create meaningful error messages:
- Stage mismatch errors
- Invalid stage transitions
- Illegal cross-stage references
- Suggestions for fixes

## Acceptance Criteria

- [ ] StageCalculator correctly analyzes simple programs
- [ ] Stage inference works for common patterns
- [ ] Cross-stage violations are detected
- [ ] Error messages clearly explain stage issues
- [ ] Performance is acceptable for large files

## Dependencies

- Step 1: Stage infrastructure
- Parser output (unified AST)
