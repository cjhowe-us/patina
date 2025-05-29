# Step 3: Stage Separation

## Objective

Implement the stage separation phase that splits the unified AST into stage-specific AST representations.

## Tasks

### 3.1 Design Stage-Specific ASTs

Create specialized AST modules:
- `stage0_ast`: Compile-time evaluation structures
- `stage1_ast`: Runtime code structures
- `stage_n_ast`: Generic higher-stage structures
- Conversion traits between representations

### 3.2 Implement StageSeparator

Build the separation algorithm:
- Traverse stage-annotated AST
- Route nodes to appropriate stage AST
- Handle cross-stage boundaries
- Maintain reference integrity

### 3.3 Cross-Stage References

Manage references between stages:
- Create placeholder nodes for splices
- Track dependencies between stages
- Generate lifting code for CSP
- Validate reference validity

### 3.4 Stage AST Optimization

Optimize stage-specific representations:
- Remove unnecessary stage information
- Compact representation for each stage
- Prepare for efficient evaluation/generation
- Minimize memory usage

## Acceptance Criteria

- [ ] AST correctly separates into stage-specific versions
- [ ] Cross-stage references are properly handled
- [ ] No information is lost during separation
- [ ] Stage-specific ASTs are optimized
- [ ] Tests cover edge cases

## Dependencies

- Step 2: Stage analysis complete
- Stage-specific AST definitions
