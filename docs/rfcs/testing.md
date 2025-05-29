# Testing Infrastructure Improvements

## Current State

Good snapshot testing coverage for parser, but lacking:
- Integration tests
- Property-based testing
- Performance benchmarks
- Fuzzing infrastructure

## Planned Improvements

### 1. Integration Tests

**End-to-End Compiler Tests**
```rust
#[test]
fn test_full_compilation() {
    let source = include_str!("programs/fibonacci.pat");
    let result = compile(source);
    
    assert!(result.is_ok());
    assert_eq!(result.diagnostics.len(), 0);
    assert!(result.has_main_function());
}
```

**Test Categories**:
- Parser → Analysis → Codegen pipeline
- Multi-file compilation
- Error recovery scenarios
- Optimization verification

### 2. Property-Based Testing

**Using proptest or quickcheck**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parser_never_panics(s in ".*") {
        // Parser should handle any input gracefully
        let _ = parse(&s);
    }
    
    #[test]
    fn type_inference_sound(ast in arb_ast()) {
        let inferred = infer_types(&ast);
        prop_assert!(check_types(&ast, &inferred));
    }
}
```

**Properties to Test**:
- Parser accepts valid programs
- Type inference is sound
- Stage levels are consistent
- Optimizations preserve semantics

### 3. Benchmark Suite

**Performance Regression Tests**
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn parse_benchmark(c: &mut Criterion) {
    let small = include_str!("bench/small.pat");
    let medium = include_str!("bench/medium.pat");
    let large = include_str!("bench/large.pat");
    
    c.bench_function("parse_small", |b| b.iter(|| parse(small)));
    c.bench_function("parse_medium", |b| b.iter(|| parse(medium)));
    c.bench_function("parse_large", |b| b.iter(|| parse(large)));
}
```

**Metrics to Track**:
- Parse time per line
- Analysis time per symbol
- Memory usage
- Compilation throughput

### 4. Fuzzing

**Find Edge Cases Automatically**
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Should not panic
        let _ = parse(s);
    }
});
```

**Fuzzing Targets**:
- Parser robustness
- Type checker soundness
- Code generator correctness
- Error recovery

## Test Infrastructure

### 1. Test Framework

**Better Organization**
```
tests/
├── unit/
│   ├── parser/
│   ├── analysis/
│   └── codegen/
├── integration/
│   ├── compile_success/
│   ├── compile_fail/
│   └── optimization/
├── property/
├── benchmarks/
└── fixtures/
    ├── programs/
    └── test_data/
```

### 2. Test Utilities

**Helper Functions**
```rust
pub mod test_utils {
    pub fn parse_file(path: &str) -> Result<AST, Error> {
        let content = fs::read_to_string(path)?;
        parse(&content)
    }
    
    pub fn assert_no_diagnostics(result: &AnalysisResult) {
        assert_eq!(
            result.diagnostics.len(), 0,
            "Unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }
    
    pub fn compile_and_run(source: &str) -> Output {
        let compiled = compile(source).unwrap();
        execute(compiled)
    }
}
```

### 3. Fixtures

**Reusable Test Data**
```rust
lazy_static! {
    static ref BASIC_PROGRAM: &'static str = include_str!("fixtures/basic.pat");
    static ref TYPE_EXAMPLES: Vec<&'static str> = vec![
        include_str!("fixtures/types/integers.pat"),
        include_str!("fixtures/types/pointers.pat"),
        include_str!("fixtures/types/functions.pat"),
    ];
}
```

### 4. Mocking

**Component Isolation**
```rust
#[cfg(test)]
mod tests {
    use mockall::*;
    
    #[automock]
    trait SymbolTable {
        fn lookup(&self, name: &str) -> Option<Symbol>;
        fn insert(&mut self, name: String, symbol: Symbol);
    }
    
    #[test]
    fn test_name_resolution() {
        let mut mock = MockSymbolTable::new();
        mock.expect_lookup()
            .with(eq("x"))
            .returning(|_| Some(test_symbol()));
            
        // Test with mock
    }
}
```

## Testing Best Practices

### 1. Test Naming
```rust
#[test]
fn parser_accepts_valid_function_declaration() { }

#[test]
fn type_checker_rejects_incompatible_assignment() { }

#[test]
fn stage_analysis_detects_invalid_stage_reference() { }
```

### 2. Test Organization
- Group related tests in modules
- Use descriptive names
- Keep tests focused
- Test both success and failure cases

### 3. Assertion Quality
```rust
// Bad
assert!(result.is_err());

// Good
assert_eq!(
    result.unwrap_err(),
    TypeError::Mismatch {
        expected: Type::Int,
        found: Type::String,
        location: Position::new(10, 5)
    }
);
```

### 4. Test Data Management
- Use fixtures for complex inputs
- Generate test data programmatically
- Version control golden files
- Document test data format

## Continuous Integration

### GitHub Actions Workflow
```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - run: cargo test
      - run: cargo test --no-default-features
      - run: cargo bench --no-run
      
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/tarpaulin@v1
      - uses: codecov/codecov-action@v1
```

## Benefits

1. **Confidence**: Comprehensive testing catches bugs early
2. **Refactoring**: Safe to make changes with good tests
3. **Documentation**: Tests serve as usage examples
4. **Performance**: Prevent performance regressions
5. **Quality**: Higher code quality through TDD