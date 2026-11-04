# Getting Started

This guide covers building and running the Patina compiler as it exists today. Today the compiler only parses: it reads one source file, builds an AST, and prints counts. For the pipeline and component status, see the [architecture overview](./architecture/overview.md). For the planned direction, see the [comptime-reflection plan](../plans/comptime-reflection/index.md).

## Prerequisites

- A stable Rust toolchain with `rustfmt` and `clippy`. CI installs the stable toolchain with both components.
- The crate uses Rust edition 2021. The binary and the library are both named `patina`.
- Dependencies come from crates.io: `hecs` 0.10.5, `pest` and `pest_derive` 2.7, `ron` 0.8, `serde` 1.0, and `thiserror` 1.0. The only dev-dependency is `tempfile` 3.8.

## Build, Run, and Check

```bash
cargo build                         # build the library and the CLI
cargo run -- path/to/file.pat       # parse one file
cargo test                          # run all unit and fixture tests
cargo fmt --all --check             # formatting check, as CI runs it
cargo clippy --all-targets --all-features -- -D warnings   # lint, as CI runs it
```

CI runs on pushes to `main` and `agent-development` and on pull requests to `main`. It runs the build, `cargo test`, `cargo test --release`, the formatting check, and clippy with warnings denied. See [testing.md](./testing.md) for the test layout.

## The CLI

The CLI takes exactly one argument, the path of a source file:

```text
patina <input-file>
```

| Situation | Output | Exit code |
|---|---|---|
| Wrong number of arguments | `Usage: <program> <input-file>` on stderr | 1 |
| The file cannot be read | `Error reading file '<file>': <error>` on stderr | 1 |
| The Pest grammar rejects the input | `Parse error: <Pest error>` on stderr | 1 |
| The input parses | The five lines shown below, on stdout | 0 |

There are no flags. The CLI does not run analysis, print the AST, or emit code.

## Example

Save this as `example.pat`:

```patina
// A small Patina program
module math {
    fun add(a: i32, b: i32): i32 {
        return a + b
    }
}

let answer = 42
let mut name = "world"
let greeting = "hello \(name)"
let numbers = [1, 2, 3]
let double = fun(n: i32): i32 { n * 2 }
let sign = if answer > 0 { 1 } else { 0 }

@inline
fun count_down(n: i32) {
    while (n > 0) {
        break
    }
}
```

Running `cargo run -- example.pat` prints:

```text
=== AST Structure ===
AST created with 70 entities
=== Parse Success ===
File 'example.pat' parsed successfully!
AST contains 8 top-level statements
```

The entity count is the size of the AST's `hecs` side table, which holds one entity per node (see [ast.md](./ast.md#the-hecs-side-table)). It is not a measure of program size that other tools rely on.

## Syntax the Parser Handles

The example uses forms that parse into meaningful AST nodes:

- `module name { ... }` blocks.
- `fun name(param: Type, ...): Return { ... }` functions and `fun(param: Type): Return { ... }` lambdas.
- `let` and `let mut` bindings with an optional type and value.
- Integer, float, string, boolean, and array literals. Strings interpolate with `\(expr)`.
- `if` / `else` expressions, `while (condition) { ... }` loops, and `return`, `break`, and `continue`.
- `//` and `/* */` comments, and `@name` or `@name(args)` annotations before a statement.

## Known Gaps

The grammar accepts more than the parser can represent. These are the gaps a new contributor is most likely to hit; [findings.md](../plans/comptime-reflection/findings.md) lists them all.

- Type syntax, such as `i32` or `*own T` in a parameter, becomes the identifier `_` (PARSE-2).
- `struct`, `enum`, `interface`, and `impl` declarations reach `todo!` and panic. `x = 5` parses as an operation rather than an assignment (PARSE-3).
- A parenthesized expression such as `(a + b)` becomes `null` (PARSE-1).
- `for` loses its header and body, and `case` produces no branches (PARSE-4, PARSE-5).
- Annotations and comments are parsed and then dropped, so `@inline` above has no effect (PARSE-8).
- Binary operators are kept as a flat list, so precedence is never resolved (AST-3).
- `PrinterVisitor` output is not valid Patina. For the example it prints `fn` instead of `fun`, `_` for every type, `${name}` for interpolation, and `\(n: _): _ =>` for the lambda, and it drops the `while` parentheses (PRINT-1).
- The legacy `static` keyword forms still parse. The plan retires them in favor of inferred comptime/runtime modes (constraint C-2). Don't write new code or fixtures that use them.

## Where to Go Next

- [Architecture overview](./architecture/overview.md): the pipeline, components, and module layout.
- [AST](./ast.md): node structures and the `hecs` side table.
- [Visitor pattern](./architecture/visitor-pattern.md): traversing the AST.
- [Testing](./testing.md): unit tests and RON fixtures.
- [Step 1 of the plan](../plans/comptime-reflection/step-1.md): the design work that comes before new compiler features.
