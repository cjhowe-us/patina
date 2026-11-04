# Testing

This guide describes the tests that exist today: Rust unit tests and parser fixtures stored as RON files. For what the tests do not yet cover, see [Known gaps](#known-gaps) and the TEST findings in [findings.md](../plans/comptime-reflection/findings.md#test--tests-and-fixtures).

## Commands

```bash
cargo test                       # all tests: 154 in the library, 2 in the binary
cargo test test_parser_valid     # one test by name
cargo test analysis::            # every test in a module
cargo test --release             # CI also runs the suite in release mode
cargo fmt --all --check          # formatting, as CI checks it
cargo clippy --all-targets --all-features -- -D warnings   # lints, as CI checks them
```

## Layout

| Location | Tests | What they cover |
|---|---|---|
| `src/analysis/*.rs` | `#[cfg(test)] mod tests` in `arena`, `context`, `diagnostics`, `id`, `stage`, `symbol_table`, and `types` | The standalone analysis types. None of them run on parser output, because analysis is not connected to the parser. |
| `src/ast/*_tests.rs` | Attached to their module with `#[cfg(test)] #[path = "..._tests.rs"] mod tests;` | Node construction (`nodes_tests.rs`, `nodes_additional_tests.rs`), the `hecs` side table (`components_tests.rs`, `mod_tests.rs`), the printer (`printer_tests.rs`, `printer_additional_tests.rs`), and visitors (`visitor_tests.rs`, `visitor_additional_tests.rs`). These build ASTs by hand. |
| `src/ast/test_utils.rs` | `test_ast_printer` | Printer output for two fixture names (see [Known gaps](#known-gaps)). |
| `src/parser.rs` | Inline `#[cfg(test)]` tests | Direct parser checks. |
| `src/parser/tests.rs` | `test_parser_valid`, `test_parser_errors` | The RON fixtures in `test_cases/parser/`. |
| `src/main.rs` | Two `#[cfg(test)]` tests | The CLI entry point. |

There are no integration tests in a `tests/` directory, no snapshot tests, and no benchmarks.

## RON Fixtures

Parser fixtures live in two directories:

- `test_cases/parser/valid/`: 78 inputs that must parse.
- `test_cases/parser/errors/`: 22 inputs that must be rejected. Each one sets `should_fail: true`.

Each file holds one `TestCase`, defined in `src/test_shared.rs`. This is `test_cases/parser/valid/simple_binding.ron`:

```ron
TestCase(
    name: "Simple binding",
    input: r#"
let x = 1
"#,
)
```

| Field | Required | Meaning |
|---|---|---|
| `name` | Yes | By convention, the file stem in sentence case (`Simple binding` for `simple_binding.ron`). It is passed to the parser as the file name, and `test_parser_valid` matches on it to choose extra assertions. |
| `input` | Yes | Patina source. Use a raw string (`r#"..."#`) so quotes and backslashes need no escaping. |
| `should_fail` | No (default `false`) | When `true`, the test passes only if parsing fails. |
| `description` | No | Free text. No test reads it. |

There is no field for the expected AST or the expected diagnostic (TEST-2, TEST-6).

## Fixture Runners

`src/test_shared.rs` is a test-only module (`#[cfg(test)]` in `src/lib.rs`) with two runners. Both read every `.ron` file in `test_cases/<directory>`, sort the files by path, deserialize each one, trim the input (TEST-3), and call the test closure with the input and the fixture name. Both panic if the directory cannot be read, holds no `.ron` files, or holds a file that cannot be read or deserialized.

- `run_directory_tests(test_name, directory, test_fn)` calls `test_fn(input, name)`. A result of `Ok` fails the test if `should_fail` is set, and `Err` fails it if `should_fail` is not set. `test_parser_errors` uses this runner.
- `run_parser_directory_tests(test_name, directory, parse_fn, verify_fn)` skips a fixture whose trimmed input is empty, printing a warning. Otherwise it parses the input, applies the same `should_fail` rule, and calls `verify_fn(name, &ast)` on a successful parse. `test_parser_valid` uses this runner.

`test_parser_valid` checks structure for only four fixture names: `Module`, `Simple binding`, `Mutable binding`, and `Static binding`. For every other fixture, `verify_fn` checks only that the AST's `hecs` world is not empty. That is always true, because the root has an entity (TEST-1). A valid fixture therefore proves that the input parses without panicking, not that it parses correctly.

## Adding a Fixture

1. Create `test_cases/parser/valid/<case>.ron` or `test_cases/parser/errors/<case>.ron`. Name the file after what the input tests, and keep the input consistent with the name (TEST-7).
2. Set `name` to the file stem in sentence case. For an error fixture, set `should_fail: true`.
3. Run `cargo test test_parser_valid` or `cargo test test_parser_errors`. The runners find new files automatically, so no Rust changes are needed.
4. If the fixture needs more than "it parses", add a match arm for its `name` in the `verify_fn` in `src/parser/tests.rs`.

Don't add fixtures that use the legacy `static` keyword. The plan retires it in favor of inferred comptime/runtime modes (constraint C-2, META-7).

## Known Gaps

- Valid fixtures mostly prove only that parsing does not fail (TEST-1). Several parser defects, such as grouped expressions becoming `null` (PARSE-1), pass the suite unnoticed.
- Error fixtures check only that parsing fails, not which error or where (TEST-6).
- `test_ast_printer` in `src/ast/test_utils.rs` selects fixtures by four file stems, but only `static_block.ron` and `function_with_static.ron` exist. These are also the only two fixtures whose `name` is the raw stem rather than sentence case. The test ignores each fixture's input and prints a mock AST built from the fixture name (TEST-4). The module also defines its own `TestCase` without `should_fail`, plus a `TestSuite` type and `load_test_suite` function that nothing uses.
- `test_main_with_invalid_input` in `src/main.rs` does not assert that the input is rejected (TEST-5).
- Many fixture names do not match their contents, and several error fixtures are duplicated or corrupted (TEST-7, TEST-8). The [fixture audit](../plans/comptime-reflection/findings.md#fixture-audit) lists each one with its planned action.
- There is no coverage for many constructs and for anything beyond parsing (TEST-10).

The test strategy that replaces this is a Step 1 design task (`testing-strategy.md`) in the [comptime-reflection plan](../plans/comptime-reflection/index.md).
