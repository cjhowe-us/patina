# Testing in Patina

Patina uses a comprehensive testing strategy combining unit tests, integration tests, and snapshot testing to ensure correctness and prevent regressions.

## Test Infrastructure

### Snapshot Testing with Insta

Patina uses the `insta` crate for snapshot testing, which:
- Captures test output as snapshots
- Compares against previous snapshots
- Provides visual diffs for changes
- Enables easy test updates

### Test Organization

```
patina/
├── src/
│   ├── parser/tests.rs      # Parser unit tests
│   ├── visitor/tests.rs     # Visitor pattern tests
│   └── */snapshots/         # Snapshot files
├── test_cases/              # Test data JSON files
└── tests/                   # Integration tests
```

## Running Tests

### Basic Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests in specific module
cargo test parser::

# Run with output
cargo test -- --nocapture
```

### Snapshot Testing

```bash
# Run with interactive review
cargo insta test --review

# Accept all snapshot changes
cargo insta test --accept

# Check without updating (CI mode)
cargo insta test --check

# Review changes from last run
cargo insta review

# Accept changes from last run
cargo insta accept
```

## Test Categories

### 1. Parser Tests

Located in `src/parser/tests.rs`, these verify:
- Correct AST generation
- Error handling
- Edge cases

Example:
```rust
#[test]
fn test_simple_binding() {
    let input = "let x = 42";
    let ast = parse(input).unwrap();
    insta::assert_snapshot!(ast);
}
```

### 2. Visitor Tests

Located in `src/visitor/tests.rs`, these test:
- Tree traversal
- Visitor callbacks
- Node counting

### 3. Walk Tests

Test the generic AST traversal:
- Pre/post order traversal
- Early termination
- Result collection

### 4. Integration Tests

Full pipeline tests including:
- Real-world examples
- Complex programs
- Error scenarios

## Test Data Files

### JSON Test Cases

Test data is stored in JSON files:

```json
{
  "SimpleBinding": {
    "input": "let x = 42",
    "description": "Basic variable binding"
  },
  "ComplexExpression": {
    "input": "let result = (a + b) * c / d",
    "description": "Arithmetic expression"
  }
}
```

### Test Categories

- `lexer.json`: Lexical analysis tests
- `parser.json`: Basic parsing tests
- `parser_error.json`: Error handling tests
- `real_world_examples.json`: Complex scenarios
- `field_access_test.json`: Member access tests

## Writing New Tests

### 1. Unit Test

```rust
#[test]
fn test_new_feature() {
    let input = "new syntax";
    let result = parse(input);
    assert!(result.is_ok());
    insta::assert_snapshot!(result.unwrap());
}
```

### 2. Add Test Case

Add to appropriate JSON file:
```json
{
  "NewFeature": {
    "input": "new syntax here",
    "description": "Tests the new feature"
  }
}
```

### 3. Generate Snapshot

```bash
cargo test test_new_feature
cargo insta review
```

### 4. Verify Output

Check the generated snapshot:
- Correct AST structure
- Proper position information
- Expected node types

## Debugging Tests

### Viewing Test Output

```bash
# See println! output
cargo test test_name -- --nocapture

# Run single test with backtrace
RUST_BACKTRACE=1 cargo test test_name
```

### Understanding Failures

1. **Snapshot Mismatch**: Run `cargo insta review` to see diff
2. **Parser Error**: Check grammar rules and input syntax
3. **Panic**: Use `--nocapture` to see debug output

### Common Issues

- **Whitespace**: Parser may be sensitive to whitespace
- **Position Info**: Line/column numbers must match
- **Node Order**: Children must be in expected order

## Test Coverage

### Current Coverage

- Parser: 100+ test cases
- AST nodes: All types covered
- Error cases: 50+ error scenarios
- Real-world: 5 complex examples

### Adding Coverage

1. Identify untested code paths
2. Add test cases to JSON files
3. Verify with snapshots
4. Add edge cases

## CI Integration

Tests run automatically on:
- Pull requests
- Main branch commits
- Release builds

CI configuration:
```yaml
- cargo test
- cargo insta test --check
- cargo clippy
```

## Best Practices

### Do's

- Write descriptive test names
- Test both success and failure cases
- Use snapshots for complex outputs
- Keep tests focused and isolated
- Document special test requirements

### Don'ts

- Don't ignore failing tests
- Don't commit without reviewing snapshots
- Don't test implementation details
- Don't duplicate test coverage
- Don't use random/time-based data

## Maintenance

### Updating Snapshots

When changing parser output:
1. Run tests: `cargo test`
2. Review changes: `cargo insta review`
3. Accept if correct: `y` for each
4. Commit snapshots with code

### Cleaning Up

```bash
# Remove unused snapshots
cargo insta test --delete-unreferenced-snapshots

# Format snapshots
cargo insta test --force-update-snapshots
```

## Performance Testing

For performance-critical code:

```rust
#[bench]
fn bench_parser(b: &mut Bencher) {
    let input = load_large_file();
    b.iter(|| {
        parse(&input)
    });
}
```

Run with: `cargo bench`