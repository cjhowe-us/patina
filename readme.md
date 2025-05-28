# Patina

Patina is a systems programming language with a focus on memory safety, modern syntax, and powerful
type system features.

## Development

### Building the code

```bash
cargo build
```

### Running unit tests

```bash
# Run all tests
cargo test

# Runs all tests and reviews any snapshot changes interactively
cargo insta test --review
```

#### Non-interactively

```bash
# Run all tests and accept any snapshot changes without interactivity
cargo insta test --accept

# Overwrites snapshots from the last test run without interactivity
cargo insta accept

# Runs the tests and does not update any snapshots, without interactivity
cargo insta test --check
```
