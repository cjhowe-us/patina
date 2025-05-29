# AST Visitor Improvements

## Overview

Create stage-aware visitors to complete the semantic analysis pipeline. These visitors will traverse the AST and perform various analysis passes.

## Required Visitors

### 1. Name Resolution Visitor

**Purpose**: Resolve identifiers to their symbol definitions

**Responsibilities**:
- Traverse AST and resolve all identifier references
- Build scope chains
- Handle shadowing rules
- Report undefined symbol errors

**Implementation**:
```rust
pub struct NameResolver<'a> {
    context: &'a mut Context,
    current_scope: ScopeId,
    errors: Vec<Diagnostic>,
}

impl<'a> NameResolver<'a> {
    pub fn resolve_identifier(&mut self, name: &str, pos: Position) -> Option<Symbol> {
        // Look up in current and parent scopes
        // Report error if not found
    }
}
```

### 2. Type Inference Visitor

**Purpose**: Infer and check types for all expressions

**Responsibilities**:
- Infer types for expressions without explicit annotations
- Verify type compatibility
- Handle type coercion
- Report type errors

**Implementation approach**:
- Use bidirectional type checking
- Maintain type constraints
- Solve constraints using unification

### 3. Stage Calculation Visitor

**Purpose**: Compute stage levels for all AST nodes

**Responsibilities**:
- Propagate stage annotations
- Infer stages for unannotated code
- Validate stage consistency
- Ensure no stage level violations

**Key rules**:
- Default stage is 1 (runtime)
- Stage 0 is compile-time
- Lower stages cannot reference higher stages

## Integration Architecture

### 1. Visitor Pipeline
```rust
pub struct AnalysisPipeline {
    visitors: Vec<Box<dyn AstVisitor>>,
}

impl AnalysisPipeline {
    pub fn analyze(&mut self, ast: &AST) -> Result<TypedAST, Vec<Diagnostic>> {
        for visitor in &mut self.visitors {
            visitor.visit(ast)?;
        }
        Ok(self.build_typed_ast())
    }
}
```

### 2. Visitor Trait
```rust
pub trait AstVisitor {
    fn visit(&mut self, node: &dyn SyntaxNode) -> Result<(), Diagnostic>;
    
    fn visit_let(&mut self, let_stmt: &Let) -> Result<(), Diagnostic> {
        // Default implementation
        self.visit_children(let_stmt)
    }
    
    // ... methods for each node type
}
```

### 3. Context Sharing
- Visitors share analysis context
- Each visitor can read/write symbols
- Diagnostics collected centrally

## AST Transformation

### Transform Trait
```rust
pub trait AstTransform {
    type Output;
    
    fn transform(&self, node: &dyn SyntaxNode) -> Self::Output;
}
```

### Stage Separation
```rust
pub struct StageSeparator {
    stage: StageLevel,
}

impl AstTransform for StageSeparator {
    type Output = Option<Box<dyn SyntaxNode>>;
    
    fn transform(&self, node: &dyn SyntaxNode) -> Self::Output {
        if node.stage() == self.stage {
            Some(node.clone())
        } else {
            None
        }
    }
}
```

## Implementation Plan

### Phase 1: Basic Visitors
1. Implement visitor trait
2. Create name resolution visitor
3. Add basic type checking
4. Test with simple programs

### Phase 2: Advanced Analysis
1. Full type inference
2. Stage calculation
3. Cross-stage validation
4. Error recovery

### Phase 3: Optimization
1. Incremental analysis
2. Parallel visitors
3. Caching results
4. Performance tuning

## Testing Strategy

### Unit Tests
- Test each visitor independently
- Mock AST nodes for testing
- Verify error detection

### Integration Tests
- Full pipeline tests
- Real program analysis
- Error message quality

### Regression Tests
- Snapshot tests for analysis results
- Performance benchmarks
- Memory usage tracking

## Benefits

1. **Modularity**: Each visitor has single responsibility
2. **Reusability**: Visitors can be reused and combined
3. **Testability**: Easy to test in isolation
4. **Extensibility**: New analysis passes are easy to add
5. **Performance**: Can be parallelized where appropriate