# Comptime Reflection Plan — Findings

> Part of the [comptime-reflection plan](./index.md). Every step of the plan cites these IDs. IDs are stable: do not renumber them. Append new findings at the end of a category.

These findings come from a read-only review of the working tree on 2026-11-04. The review covered all 27 Rust files, 27 Markdown files, 100 RON fixtures, and the Pest grammar. At review time, `cargo test --lib` reported 154 passed and 0 failed. The passing suite does **not** verify most of the behavior below (see TEST-1).

Behavior marked **(probed)** was confirmed by running `target\debug\patina.exe` on a one-line input file. Line numbers refer to the working tree at review time and may drift.

Some cited documents have since been deleted. Those paths and line numbers are historical evidence, not required inputs; see [Removed documentation](#removed-documentation). Finding IDs and still-relevant compiler requirements are retained.

## Design owners

Each finding is assigned to the Step 1 design document that must address it. A finding can be resolved by a design decision, or explicitly deferred with a reason.

| Key | Design document (to be written in `docs/designs/`) |
|---|---|
| ARCH | `compiler-architecture.md` |
| CT | `comptime.md` |
| REFL | `static-reflection.md` |
| INCR | `incremental-compilation.md` |
| IREFL | `incremental-reflection.md` |
| SYN | `syntax-and-parsing.md` |
| ASTIR | `ast-and-ir.md` |
| SEM | `semantic-analysis.md` |
| TST | `testing-strategy.md` |

## PARSE — Parser conversion (`src/parser.rs`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| PARSE-1 | Grouped expressions are replaced with `Null`. The parser passes the `left_paren` token instead of the inner expression, and the childless token reaches a fallback that invents `Literal::Null`. `Expression::Grouped` is never constructed. | `parser.rs:842`; fallback `parser.rs:456–462`. Fixtures `grouped_expression`, `nested_expressions`, and `complex_nested_expressions` still pass. | SYN, ASTIR |
| PARSE-2 | Type syntax is replaced with the identifier `_`. The type helper unwraps the `atomic_type`/`type_expression` wrapper only once, so it sees `type_expr` and matches neither primitive arm. Pointer, array, and function types are never built. This affects parameters, return types, binding annotations, and types used as values. | `parser.rs:933–960` | SYN, ASTIR |
| PARSE-3 | `struct`, `enum`, `interface`, `impl`, and `assignment_stmt` are accepted by the grammar but reach `todo!` in statement dispatch. **(probed)** `struct S {}` panics at `parser.rs:206`. | `parser.rs:188–206`; grammar `grammar.pest:182, 191–207` | SYN, CT |
| PARSE-4 | `for` returns a `For` node with no initializer, condition, update, or body. | `parser.rs:411–429`; grammar `grammar.pest:186–188` | SYN |
| PARSE-5 | `case` returns a synthetic `Null` scrutinee and no branches. | `parser.rs:554–567`; see GRAM-10 | SYN, ASTIR |
| PARSE-6 | Numeric conversion uses `parse().unwrap()` and panics when an integer is out of range (for example `9223372036854775808`) or when a span contains whitespace (see GRAM-1). | `parser.rs:869` (integer), `parser.rs:875` (float) | SYN, ASTIR |
| PARSE-7 | Defensive `todo!` arms remain in expression, atom, and atomic-value dispatch. They must become internal-error diagnostics or be proven unreachable. | `parser.rs:476, 855, 886` | SYN |
| PARSE-8 | Annotations and leading comments are parsed into local vectors and then discarded. Only the statement is returned. | `parser.rs:110–132`; annotation arguments are built at `parser.rs:135–150` | SYN, CT |
| PARSE-9 | The parser's general pattern on unexpected shapes is to invent values (`Null`, `_`) and report success, instead of returning a positioned error. This violates constraint C-5. | PARSE-1, PARSE-2, PARSE-5 | SYN |

## GRAM — Grammar (`src/grammar.pest`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| GRAM-1 | Identifier, number, and string rules are non-atomic, so implicit `WHITESPACE`/`COMMENT` skipping happens *inside* tokens. **(probed)** `let x = ab cd` is accepted as a single identifier. | `grammar.pest:35–53` | SYN |
| GRAM-2 | Keywords and type names have no token boundary at their use sites. With PEG ordered choice, this either merges tokens or rejects valid identifiers. **(probed)** `letx = 1` parses as `let x = 1`, and `let x: *mutable` parses as `*mut able`. `let x = true_value` and `let x = i32_max` are rejected because `true` or `i32` matches first. Because `atomic_type` is tried before `binding`, identifiers such as `f1`, `i8`, and `u2` are classified as types. | keywords `grammar.pest:11–32`; `ident` 39; `integer_type`/`floating_type` 116–117; `pointer_type` 119; `atom` order 135 | SYN |
| GRAM-3 | `atomic_value` tries `integer` before `float`, so `3.14` is not parsed as a float literal. **(probed)** `let x = 3.14` produces more AST entities than `let x = 3`. | `grammar.pest:132` | SYN |
| GRAM-4 | Strings have no escape sequences. `string_fragment` excludes `\`, and `\(` starts interpolation. **(probed)** `let x = "a\nb"` is rejected. | `grammar.pest:50–53` | SYN |
| GRAM-5 | `label` and `tag` are non-atomic and accept bare `ident_char+`, so labels such as `:1` are accepted and whitespace can be skipped inside them. | `grammar.pest:41–42` | SYN |
| GRAM-6 | Dead alias rules (`*_statement`, `call`, `index`, `value`, `case_arm`) and an unused `comment` rule. The parser uses only `Rule::field`. | `grammar.pest:214, 217–241`; `parser.rs:811` | SYN |
| GRAM-7 | Generic parameter lists `<T, ...>` on `struct`, `enum`, `interface`, and `impl` contradict constraint C-1 (no generics). | `grammar.pest:191–207` | CT, SYN |
| GRAM-8 | The legacy `static_kw` forms (`static let`, `static fun`, `static { }`) are retired with no replacement keyword or annotation (C-2). Comptime-ness is inferred from calls to built-in comptime intrinsics (META-11), so no grammar marks comptime code. General annotation syntax is unaffected. | legacy forms `grammar.pest:22, 32, 170, 178, 180` | CT, SYN |
| GRAM-9 | `for … in` uses a literal `"in"`, which is not a keyword and has no boundary. | `grammar.pest:187` | SYN |
| GRAM-10 | The `case` grammar has no scrutinee (`case_branch = case_kw ~ expression ~ expression`), but the AST requires `Case.value` plus pattern and guard branches. | `grammar.pest:151–152`; `nodes.rs:253` (`Case`), `261` (`CaseBranch`) | SYN, ASTIR |
| GRAM-11 | There is no quote, splice, or reflection syntax. The removed stage design used `@[stage(n)]` and `$()`, which do not match the grammar's `@name(args)` annotation syntax. The new design must decide whether ordinary intrinsic calls suffice; the old syntax is not a requirement. | `grammar.pest:215`; `docs/designs/stage-levels.md:23–53, 152–168` | SYN, REFL |
| GRAM-12 | `assignment_stmt = expression "=" expression` is ambiguous with `=` as a binary operator. In practice `x = 1` is parsed as an operation. | `grammar.pest:182`; `operator` `grammar.pest:102` | SYN, ASTIR |
| GRAM-13 | A `;` separator in a block must be followed by another statement, so `{ let x = 1; }` is rejected. The root rule does not accept `;` at all. | `grammar.pest:246–249` | SYN |
| GRAM-14 | Backtick identifiers (`` `-path` ``) are called "quoted" in fixture names. This collides with the planned meaning of *quote* (code quotation). | `grammar.pest:38`; fixtures `quoted_*` | SYN, TST |
| GRAM-15 | The AST has `Literal::Null`, but the grammar has no `null` literal. `return null` parses `null` as an identifier. | `nodes.rs:382` (`NullLiteral`); fixture `function_with_multiple_parameters_of_different_types` | SYN, ASTIR |
| GRAM-16 | Comments are implicitly skipped by the silent `COMMENT` rule, but `decorated_statement` also tries to match them explicitly. Which comments are retained is therefore unclear, and there is no doc-comment syntax. | `grammar.pest:6–8, 243` | SYN |

## AST — Syntax tree (`src/ast/`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| AST-1 | The `Annotation` component stores `Vec<String>` rather than parsed argument expressions with spans. | `components.rs:59–64` | ASTIR, CT |
| AST-2 | `Statement::Static`, the `Static` node, and the `is_static` fields encode the legacy keyword forms. Under C-2 they are removed, not renamed. Comptime/runtime mode becomes a compiler-derived, per-item fact (META-11), not an AST node or flag. | `nodes.rs:40, 55, 76, 101, 188–190` | ASTIR, CT |
| AST-3 | `Operation.elements` is a flat, source-ordered operand/operator list. Precedence is never resolved, and no design says where it is resolved or whether operators are comptime-extensible. | `nodes.rs:283–295` | ASTIR, CT |
| AST-4 | Integer literals are stored as `i64`. `u64` values above `i64::MAX` cannot be represented, and the literal's spelling is lost. | `nodes.rs:347–349` | ASTIR |
| AST-5 | There is no representation for executable code fragments or reflection/reification values. The designs must determine whether quotation and splicing need new nodes or whether ordinary calls to comptime intrinsics suffice. The absence of any comptime-marker syntax node is intended (C-2). | `src/ast/nodes.rs`; META-2, META-11 | ASTIR, REFL |
| AST-6 | Two stage types exist: the ECS `Stage` component and `analysis::Stage`. Both are replaced by the concepts that ANL-10 defines. | `components.rs:33–35`; `analysis/stage.rs` | ASTIR, CT |
| AST-7 | Entity IDs are local to a `hecs::World` allocation and are copied by `Clone`. They are not stable across parses. These are the AST's vestigial side-table entities (AST-9). | `ast/mod.rs:64–82` | INCR, ASTIR |
| AST-8 | There is no index of item or module ownership. A module is a name plus inline statements. | `nodes.rs:62` (`Module`) | INCR, ARCH |
| AST-9 | The AST is **not** an ECS structure, despite earlier documentation. It is a plain owned Rust tree: `Root.statements` is a `Vec<Statement>`, and every node owns its children inline through `Vec`, `Box`, and `Option`. `Ast` also holds a `hecs::World`, and each node carries an `entity: hecs::Entity` spawned empty, but the tree structure is not in the world. The parser attaches only `Position` to those entities, and some synthetic entities get nothing. No other component is attached outside tests. Only the `FilterTraverser` helpers, the CLI's entity count, and tests read the world. The printer, the `Visitor` walk, and `src/analysis/` never touch it. The world is therefore a vestigial span side table, so per-node entity allocation buys nothing that an inline span field would not. `ast-and-ir.md` decides whether to remove it. The recommended direction is to store the span inline on each node (together with POS-6) and drop `Ast::world`, `component_entities`, the `Ast<C>` marker parameter, and the per-node `entity` fields, so that only the IR uses `hecs` (C-8). Facts belong to queries (INCR) or IR components, not to AST entities. | `ast/mod.rs:35–44, 63–118`; `nodes.rs` (`Root`, `entity` fields); `parser.rs:62, 154–186`; `visitor.rs:341, 350`; `main.rs:33` | ASTIR, SYN, INCR |

## PRINT — Source printer (`src/ast/printer.rs`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| PRINT-1 | The printer emits a different surface language: `fn` instead of `fun`, trailing semicolons, `\(…) => …` lambdas, apostrophe labels, `while` without parentheses, `${}` interpolation (which re-parses as plain text), and string escapes the grammar rejects. | `printer.rs:110–164, 195–217, 432–450, 511–532` | SYN, TST |
| PRINT-2 | Printer tests assert this incompatible output. | `printer_tests.rs:157–159, 212–214`; `printer_additional_tests.rs:30–32, 122–124` | TST |

## VIS — Traversal (`src/ast/visitor.rs`, `src/ast/mod.rs`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| VIS-1 | `ComponentFilterWrapper` does not forward `visit_expression` or `visit_literal`, so wrapped visitors silently lose those callbacks even when the filter always passes. | trait methods `visitor.rs:21, 28`; wrapper `visitor.rs:357, 372–544` | ASTIR |
| VIS-2 | `Traversable<C>` and `GenericVisitor<C>` have no implementations for any AST node. The default `accept` does not recurse. | `ast/mod.rs:120–136`; `visitor.rs:310–317` | ASTIR |
| VIS-3 | Traversal is preorder with unconditional recursion, has no enter/exit hooks, and receives only `&Ast`. Scoped resolution and fact attachment cannot be built on it. | `visitor.rs:85–105, 274–280` | ASTIR, ARCH |

## POS — Source provenance

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| POS-1 | Positions created during conversion always have `filename: None`. | `parser.rs:179` | SYN, INCR |
| POS-2 | Call, index, and field nodes have no position. | `parser.rs:754–825` | SYN |
| POS-3 | The CLI calls `parse_file` instead of `parse_file_with_name`, even though it knows the filename. | `main.rs:14–24` | ARCH |
| POS-4 | Diagnostic rendering ignores `Position.filename`. | `analysis/diagnostics.rs:170–200` | ARCH |
| POS-5 | There is no `FileId`, revision-aware span, or generated-code provenance (expansion site plus definition site). Location data is not separated from semantic fingerprints. | — | INCR, REFL |
| POS-6 | Source locations record only a start point, so they cannot describe a construct. `Position` holds the start `line` and `column` plus a byte `offset` and `length`; there is no end line or end column. `ParseError` has only a start `line`/`column`, and diagnostic rendering prints only the start. Locations must become **ranges** from a start line/column to an end line/column that cover every character in between, so that diagnostics can report and underline whole constructs, including multi-line ones. The design must decide: whether the end is exclusive; the column unit (bytes, Unicode scalar values, or UTF-16 code units for editor protocols); whether byte offsets are kept alongside line/column; zero-width and multi-line ranges; ranges for synthetic and generated nodes (POS-5); conversion from the Pest `Span`, whose end position supplies the end line/column; rendering; the `FileId` that a range belongs to; and keeping ranges out of semantic fingerprints. With AST-9, the range is stored inline on each node. | `components.rs:9–29`; `parser.rs:17–27, 62–70, 154–186`; `analysis/diagnostics.rs:13, 182`; `analysis/symbol_table.rs:33` | SYN, ASTIR, ARCH, INCR |

## ANL — Semantic analysis (`src/analysis/`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| ANL-1 | The type-ID domains are incompatible. Symbols store `Id<TypeTag>`, while the registry returns `Id<Type>`. | `id.rs:48–56`; `symbol_table.rs:29–35, 56–59`; `types.rs:177–185` | ASTIR |
| ANL-2 | Stage shadowing is inconsistent. `lookup_at_stage` stops at the nearest inaccessible declaration, while `lookup_all_visible_stages` (and its test) falls back to an outer one. | `symbol_table.rs:247–255` vs `636–668` | CT |
| ANL-3 | `Stage::can_see_symbol_at` lets later stages see earlier-stage symbols, but this is not value availability. There is no lifting or serialization, no prevention of comptime pointers escaping, and no runtime-callability rule. | `stage.rs:49–53` | CT |
| ANL-4 | `TypeChecker` checks only explicitly supplied code-type and stage relationships. There is no resolver, expression type checking, inference, ownership checking, or control-flow validation. | `types.rs:313–355` | ASTIR, ARCH |
| ANL-5 | `AnalysisContext` forks share the current stage and the symbol table's mutable scope cursor. | `context.rs:60–78, 95–103` | INCR, ARCH |
| ANL-6 | `Type::TypeParameter` and `Type::Generic` contradict constraint C-1. | `types.rs:56, 58` | CT, ASTIR |
| ANL-7 | `Type::Code`, `StagedType`, `check_splice`, and `check_quote` exist but are not connected to any pass. | `types.rs:59–60, 94, 332–353` | CT, REFL |
| ANL-8 | `ErrorCollector` is a mutable accumulator that is drained destructively. Diagnostics are not owned by the computation that produced them. | `diagnostics.rs:75, 103, 107` | INCR |
| ANL-9 | `Arena<T>` is unneeded and should be replaced. It is an append-only `Vec<T>` that returns `Id<T>` indices. Its only users are `SymbolTable` (scopes and symbols) and `TypeRegistry` (types), neither of which is connected to the parser. It duplicates the handle-and-store role that the `hecs`-backed IR is required to fill (C-8), and has no removal or stale-handle check. Its indices depend on insertion order, so they cannot key incremental queries (INC-2). A second, incompatible ID family (the tag aliases `SymbolId`, `TypeId`, and `ScopeId`) exists alongside it and causes ANL-1. Replace `Arena<T>`, `Id<T>`, and the tag aliases with one identity scheme: stable query identities, `hecs` entities for allocation-local storage, and a type interner if the design needs one. | `arena.rs:3–49`; `id.rs:4–56`; `symbol_table.rs:7–8, 81, 150, 181`; `types.rs:17, 185–196` | ASTIR, INCR |
| ANL-10 | "Stage" conflates three concepts and needs to be rethought and renamed. `analysis::Stage(u32)` hard-codes a numbered level, with `COMPILE_TIME = 0` and `RUNTIME = 1`. The ECS `Stage` component duplicates it (AST-6), and the symbol table, analysis context, and type registry thread a "current stage" through lookup and types. Under C-2 and Goal 4 the plan needs three separate concepts: (1) **mode**, whether a function runs at comptime, runtime, or both, which is an inferred per-function fact (META-11); (2) **sealing**, whether an item is fully reified, so that its facts can no longer change and later comptime code may reflect it (META-9); and (3) **reification order**, a partial order derived from the reflects-on and reifies edges between items, in which an item's comptime code runs only after the items it reflects are sealed, and cycles are diagnosed. A global integer level is at most a display artifact of that order. The name is also ambiguous: "stage" collides with compiler pipeline stages (parse, resolve, lower), and "phase" collides with the removed phase-separation plan. `comptime.md` must separate the concepts and name each one. Candidates for (3) include *round*, *layer*, *tier*, *level*, *generation*, and *epoch*, or no name at all if only the order matters. Record the rejected names and their collisions. Until then, "stage" in this plan is a provisional legacy term. `Stage`, `declare_staged`, `lookup_at_stage`, `lookup_all_visible_stages`, `Type::Code { from_stage }`, `StagedType`, and the stage scopes are to be replaced or removed accordingly. | `analysis/stage.rs:3–54`; `components.rs:33–35`; `analysis/symbol_table.rs:223–276`; `analysis/context.rs:18, 55, 70`; `analysis/types.rs:60, 94, 313–353` | CT, IREFL, INCR, ASTIR |
| ANL-11 | There is no semantic-analysis design, and Patina needs more analysis than most languages, beyond what even Rust's compiler performs. Besides name resolution and type checking (ANL-4), the compiler must: check ownership, aliasing, mutability, and lifetimes for the four pointer kinds (`*own`, `*raw`, `*mut`, `*`); infer each function's mode (META-11); check types computed by comptime functions, which interleaves comptime evaluation with type checking because types are comptime values (C-1); check that generated code is well formed and hygienic (META-3, META-4); compute reachability for each mode (META-12); track effects through RVSDG state edges, including tracked comptime inputs (C-6); order reflection and reification (ANL-10, META-9); and invalidate all of these incrementally across reification order. `src/analysis/` performs none of these on real programs. A design must define each analysis as a query over resolved items or the RVSDG: its inputs and outputs, its fixed-point strategy (for example, over call-graph strongly connected components or region dataflow), the order in which analyses run and which ones interleave, which facts reflection may expose, how diagnostics carry ranges (POS-6), and performance budgets. | `src/analysis/`; ANL-4, META-3, META-11, META-12, C-6, C-9 | SEM, ASTIR, INCR |

## IR — Intermediate representation and driver

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| IR-1 | There is no IR, no lowering, and no backend. C-8 requires an RVSDG represented with `hecs` entities and components. The IR will be the compiler's first real ECS structure; the AST is a plain tree (AST-9). The design must cover resolved references, runtime versus type/meta values, operations, nested regions and ports, value/state dependencies, boundaries between reification-ordered items (ANL-10), validation, and stable identities for incremental reuse. | `lib.rs:9–17`; C-8 | ASTIR, INCR |
| IR-2 | The executable is a single-file parse demo. It prints entity and statement counts and exits, with no compiler session or pipeline. | `main.rs:14–40` | ARCH |

## INC — Incremental compilation

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| INC-1 | There is no compiler database or session. Every parse builds a fresh AST world. | `parser.rs:54–59` | INCR, ARCH |
| INC-2 | Identities depend on allocation. `Arena::insert` uses the current vector length, and ECS entities are world-local. | `arena.rs:13–17`; `ast/mod.rs:64–82` | INCR |
| INC-3 | There are no fingerprints, recorded dependencies, or revisions. | — | INCR |
| INC-4 | There is no module or import graph, and no file-to-module ownership. | — | INCR, ARCH |
| INC-5 | The only cache is a per-registry structural type interner. It is not incremental. | `types.rs:185–208` | INCR |
| INC-6 | Planned comptime file I/O has no tracked-input contract. | `docs/plans/phase-separation/step-4.md:35–41` | INCR, REFL |
| INC-7 | Diagnostics are not produced per query and cannot be replaced or remapped per revision. | ANL-8 | INCR |
| INC-8 | The removed performance and AST visitor RFCs deferred incremental and query architecture until late optimization phases. Incremental foundations must instead precede semantic analysis; removing those RFCs does not implement that foundation. | `docs/rfcs/performance.md:142–145`; removed AST visitor RFC, Implementation Plan | INCR |

## META — Comptime metaprogramming and reflection

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| META-1 | There is no comptime evaluator or interpreter. | — | CT, ASTIR |
| META-2 | There are no reflection values (type values, declaration handles, field handles) and no operations that expose compiler facts to comptime code. | — | REFL |
| META-3 | There is no reification and no loop that reanalyzes generated code. | — | REFL, ARCH |
| META-4 | The removed staging RFC deferred hygiene to "advanced features"; it is needed from the first generated binding and remains unimplemented. | `docs/rfcs/staging-features.md:161–165` | REFL |
| META-5 | There is no identity scheme for generated items. Stable identity is needed for incremental reuse, diagnostics, and removal of stale items. | — | REFL, INCR, IREFL |
| META-6 | Generics are planned in docs and grammar. Under C-1 they must be replaced by comptime functions that take and return types, including for parameterized `struct`/`enum`/`interface`/`impl`. | GRAM-7, ANL-6, DOC-8 | CT |
| META-7 | The legacy `static` markers in fixtures, the AST, and the docs must be retired without replacement syntax (C-2). Fixtures that exercised `static` must be rewritten so that comptime evaluation follows from calls to comptime intrinsics (META-11), or replaced. | GRAM-8, AST-2; fixtures `static_*`, `function_with_static` | CT |
| META-8 | Annotations are parsed but erased before semantic processing (PARSE-8). No contract defines what an annotation means or how it attaches to its target. Annotations carry no comptime/runtime meaning (C-2), but comptime code may need to reflect on them. | PARSE-8, AST-1 | CT, REFL |
| META-9 | There is no model of item sealing ("fully reified"), no reification order that guarantees comptime reflection sees only sealed items (formerly "stage *n + 1* sees completed stage *n*"), and no invalidation along that order. ANL-10 separates these concepts from mode and renames "stage". | — | IREFL |
| META-10 | Undecided: does "replace language features" include user-defined surface syntax (token or CST macros)? Typed reflection and reification alone do not allow new syntax. | — | CT (open question) |
| META-11 | There is no call graph and no comptime/runtime mode inference (C-2). The design must define: the set of built-in comptime intrinsics; propagation from a function that calls an intrinsic to its transitive callers; dual-mode functions; recursion and call-graph cycles; indirect calls through function values and through reified code; what triggers comptime evaluation when nothing marks it (for example, a type position, or a value that a comptime-only call requires); which comptime results can be lifted into runtime code, and how; the diagnostic when runtime code would call a comptime-only function; how the bindings and blocks formerly written `static let` and `static { }` are expressed; and how modes relate to reification order (ANL-10). Mode must be an incremental per-item fact that is invalidated only when a callee's mode actually changes. | `stage.rs:49–53`; ANL-3, GRAM-8 | CT, INCR, IREFL |
| META-12 | There is no dead code elimination. The comptime and runtime outputs must each be pruned to what their roots reach, so unused versions of dual-mode functions, and comptime-only functions, do not affect the executable binary. The design must define the roots of each mode (for example, the entry point and exported items for runtime, and evaluation sites for comptime), whether reflecting on an item makes it reachable in either mode, and how reified items become reachable. Reachability should be an incremental query. | IR-1 | CT, ASTIR, INCR |

## TEST — Tests and fixtures

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| TEST-1 | 74 of the 78 valid fixtures only check that the AST world is nonempty, which the root entity always guarantees. | `src/parser/tests.rs:18–77` | TST |
| TEST-2 | The RON schema has no expected-structure field. | `src/test_shared.rs:10–15` | TST |
| TEST-3 | Both directory runners trim the input, which hides file-boundary trivia. | `src/test_shared.rs:46, 97` | TST |
| TEST-4 | The printer fixture test ignores `test_case.input` and builds a mock AST from the name. | `src/ast/test_utils.rs:81–110` | TST |
| TEST-5 | The binary's invalid-input test does not assert rejection. | `src/main.rs:65–74` | TST |
| TEST-6 | Error fixtures assert only failure, not diagnostic kind or location. | `test_cases/parser/errors/*.ron` | TST |
| TEST-7 | Many fixture names do not match their contents. | [Fixture audit](#fixture-audit) | TST |
| TEST-8 | Three error fixtures are identical and corrupted, and another is corrupted. | [Fixture audit](#fixture-audit) | TST |
| TEST-9 | `insta` snapshot testing was documented but is not a dependency, and there are no snapshot files. The docs no longer describe it; whether to adopt it is open. | `Cargo.toml`; DOC-5 | TST |
| TEST-10 | There is no fixture coverage for: float literals, string escapes, `while`, `for`, `case`, `struct`/`enum`/`interface`/`impl`, assignment, keyword or type-name boundaries, integer overflow, positions and filenames, comptime inference (direct, transitive, dual-mode, cycles, and a runtime call to a comptime-only function), dead code elimination of each mode, comptime semantics, generated code, or incremental equivalence (including mode-change invalidation). | `test_cases/parser/` | TST |
| TEST-11 | CI failed at `HEAD`. The workflow runs `cargo clippy --all-targets --all-features -- -D warnings` on the current stable toolchain, which (rustc 1.98.1 at review time) reported `mismatched_lifetime_syntaxes` for three `Ast` accessors whose return types (`hecs::Ref<T>`, `hecs::RefMut<T>`, `hecs::QueryBorrow<Q>`) hid an elided lifetime, and `-D warnings` turned that into an error. | `src/ast/mod.rs`; `.github/workflows/ci.yaml` | **Resolved** by writing the lifetimes as `'_`; the clippy and `cargo fmt --check` commands from CI now pass. Keeping them passing is a requirement of every step |

## DOC — Documentation

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| DOC-1 | The status page claims a complete parser with full AST generation, error recovery, and positions for all nodes; lists generics as "not yet supported", implying they are planned; and lists the parser, analysis, and stage system as stable and complete. Its AST claims were corrected by the AST-9 fix. | `docs/status/current.md:11–16, 24–30, 80, 87–92` | all; WA-9 |
| DOC-2 | The stage-levels design described implemented stage inference and used `@[stage(n)]`, `$()`, `fn`, generics, and stage polymorphism. | `docs/designs/stage-levels.md:5, 75–81, 135, 194` | **Resolved** by deleting the superseded design; the replacement design remains a Step 1 task |
| DOC-3 | The visitor-pattern doc described a nonexistent `SyntaxNode`/walk API and `src/ast/walk.rs`. | Former `docs/developer/architecture/visitor-pattern.md:15–67` | **Resolved** by rewriting the doc against `src/ast/visitor.rs` and `src/ast/mod.rs`; the API gaps themselves remain as VIS-1 to VIS-3 |
| DOC-4 | The performance RFC quoted analysis throughput numbers with no pipeline or benchmark behind them. | `docs/rfcs/performance.md:77–80` | **Resolved** by deleting the invalid RFC |
| DOC-5 | The testing guide documented `insta` workflows that do not exist. | Former `docs/developer/testing.md:9, 49–61, 79, 141` | **Resolved** by recreating the guide from the current tests and fixtures; whether to adopt snapshot testing remains TEST-9 |
| DOC-6 | Getting-started used `fn`/`->`, `@[stage(0)]`, `$()`, and `SyntaxNode`. | Former `docs/developer/getting-started.md:118, 134, 138, 156` | **Resolved** by recreating the guide with verified commands, CLI output, and syntax the parser accepts (`fun`, `:` return types) |
| DOC-7 | The component-based AST doc described `Ast<C>` and `GenericVisitor` usage that has no node implementations (see VIS-2), and presented the AST as an ECS structure (AST-9). | `docs/developer/ast.md` | **Resolved** by rewriting the doc as `ast.md`, which describes the AST as a plain tree with a vestigial `hecs` side table |
| DOC-8 | The old language proposals specified generics, contradicting C-1, and the developer docs did not distinguish Rust implementation generics from forbidden Patina language generics. | Removed: `docs/rfcs/type-system.md`, `staging-features.md`, `codebase.md`, `performance.md`; `docs/designs/stage-levels.md`; `docs/plans/phase-separation/step-5.md`. Current: `overview.md:55, 63`; `ast.md:23`; `visitor-pattern.md:124` | **Resolved**: the proposals were removed, and the developer docs now identify `Ast<C>` and `GenericVisitor<C>` as Rust marker parameters and `Type::TypeParameter`/`Type::Generic` as conflicting with C-1 (ANL-6). The status page's generics limitation remains under DOC-1 |
| DOC-9 | `docs/README.md` links to nonexistent `user/`, `user/getting-started.md`, `status/component-based-ast-implementation.md`, and `status/IMPLEMENTATION_COMPLETE.md`. | `docs/README.md` | **Resolved** by the plan-setup change |
| DOC-10 | `AGENTS.md` (renamed from `CLAUDE.md`) said "Parser: Complete" and "basic type checking and symbol resolution", documented `insta` commands and snapshot testing, linked a nonexistent `docs/user/`, and pointed to the phase-separation plan. It also said "All nodes implement `SyntaxNode`" and cited nonexistent `src/visitor.rs` and `src/ast/walk.rs`. | `AGENTS.md` | **Resolved** by the plan-setup change, the AST-9 documentation fix, and the developer-docs validation. The legacy stage description remains, marked provisional under ANL-10 |
| DOC-11 | Phase-separation step 3 required duplicated per-stage ASTs, which conflicts with item-level queries. | `docs/plans/phase-separation/step-3.md:11–18` | **Resolved** by deleting the superseded plan; the query-based replacement remains in the carry-over table |
| DOC-12 | `docs/rfcs/codebase.md` was not linked from the docs index. | `docs/README.md` | **Resolved** by the plan-setup change; the RFC and its index entry have since been removed |
| DOC-13 | The codebase RFC linked to a nonexistent `../development/getting-started.md`; the guide is at `../developer/getting-started.md`. | `docs/rfcs/codebase.md:137` | **Resolved** by deleting the invalid RFC |
| DOC-14 | The readme, `AGENTS.md`, `docs/README.md`, the status page, the architecture overview, and a root `AST_IMPLEMENTATION_SUMMARY.md` described the AST as a component-based ECS structure, and the planned IR as `hecs`-backed "like the AST". The AST is a plain tree (AST-9). | `readme.md:3, 7, 16`; `AGENTS.md:60`; `docs/README.md:5–10`; `docs/status/current.md:7, 18–24`; `overview.md:13, 33, 46–50` | **Resolved** by the AST-9 documentation fix; the summary file was deleted |

### Removed documentation

Removed on 2026-11-04 at the maintainer's request. These are historical paths, not links to current inputs. Git history retains the original documents.

| Removed document | Why it no longer applies | Relevant work retained |
|---|---|---|
| `docs/designs/stage-levels.md` | Superseded stage markers, generics, and stage polymorphism conflict with C-1 and C-2. | GRAM-11, DOC-2, DOC-8; `comptime.md` and `static-reflection.md` design tasks |
| `docs/plans/phase-separation/` (index and six steps) | The pass-based, per-stage AST roadmap is replaced by incremental per-item queries. | [Phase-separation carry-over](./index.md#phase-separation-carry-over), INC-6, DOC-11 |
| `docs/rfcs/type-system.md` | Generic parameters, monomorphization, and conventional trait/associated-type primitives conflict with C-1 and the plan's non-goals. | ANL-4, ANL-6, META-6, DOC-8; type checking and type-valued comptime functions |
| `docs/rfcs/staging-features.md` | Explicit stage markers and stage polymorphism conflict with inferred modes; hygiene cannot be deferred. | ANL-3, ANL-7, META-4, META-9; lifting, reification, and stage ordering |
| `docs/rfcs/ast-visitors.md` | Whole-AST passes with shared mutable context, central diagnostics, per-stage AST splitting, and late incrementality conflict with the query-based pipeline. | VIS-1 to VIS-3, ANL-4, ANL-5, ANL-8, IR-1, INC-8; traversal and analysis over the required ECS-backed RVSDG |
| `docs/rfcs/performance.md` | Incrementality is deferred until the final phase, throughput claims are unmeasured, and generic monomorphization conflicts with C-1. | INC-1 to INC-8, META-12, DOC-4; query reuse and dead code elimination |
| `docs/rfcs/codebase.md` | The aggregate roadmap repeats generic types, stage polymorphism, and stage-separated ASTs as implementation priorities. | The active plan's work areas and findings; still-applicable error-handling, documentation, testing, and developer-experience RFCs |
| `AST_IMPLEMENTATION_SUMMARY.md` (repository root) | Presented the AST as an ECS structure with generic visitors; nothing linked to it. | AST-9, VIS-2, DOC-14 |

The remaining RFCs are inputs for problem discovery, not approval of their sample APIs or implementation order. Step 1 must reconcile them with the active constraints before they inform implementation plans.

## Fixture audit

All 100 parser fixtures in `test_cases/parser/` (78 in `valid/`, 22 in `errors/`) were read. The dispositions below are **proposals**. `testing-strategy.md` (Step 1) decides them, and an implementation step created by Step 2 applies them. In this table, "Matches" means the content matches the name. It does **not** mean the parser handles the fixture correctly; most valid fixtures still lack structural assertions (TEST-1).

Disposition codes:

- **Keep**: no change beyond adding structural expectations.
- **Rename**: rename the file and `name` to match the content.
- **Rewrite**: change the input so it tests what the name says.
- **Comptime**: retire the legacy `static` markers (C-2). Do not introduce a `comptime` keyword or a `@comptime` annotation. Rewrite the input so that comptime evaluation follows from a call to a comptime intrinsic, as `comptime.md` defines, and rename the fixture to describe what it tests. Because no syntax marks comptime code, the parser fixture may become an ordinary-syntax case, with the comptime behavior asserted by semantic tests (`testing-strategy.md`).
- **Merge**: consolidate with an overlapping fixture.
- **Replace**: delete, and create a correct fixture for the intended case.
- **Revisit**: semantically questionable; decide once semantics exist.

### Valid fixtures whose content does not match the name

| Fixture | Actual input (abridged) | Problem | Proposed disposition |
|---|---|---|---|
| `binding` | ``my.`-path` `` | A path-expression statement, not a binding. | Rename (for example `backtick_identifier_path`), or Merge with `quoted_binding`. |
| `complex_indexing_chain` | `let x = arr[idx1][idx2][fn()+3].member()` | Uses `fn` as an identifier, which is confusing next to the `fun` keyword. | Rewrite `fn()` to an ordinary name. |
| `doc_comment` | `/* My doc comment */ fun test() { … }` | An ordinary block comment; no doc-comment syntax exists (GRAM-16). | Rename to `block_comment_before_function`, or Rewrite once doc-comment syntax is designed. |
| `function_with_complex_return_type` | `fun getCallback(): *i32 { … }` | The return type is a simple pointer. | Rewrite with a compound type (for example a pointer to an array of owned pointers, or a function type). |
| `integer_width_edge_cases` | `let min: i8 = 0`, `let max: u64 = 0` | No edge values. | Rewrite with real bounds after AST-4 and PARSE-6 are designed. Add overflow error fixtures. |
| `lambda_expression` | `let adder = fun (b: i32) { let a = 1 + b }` | The lambda named `adder` returns nothing. | Rewrite the body to return its result. |
| `module` | `module test { let x = *mut i32 }` | Generic name; binds a type value. | Rename (for example `module_with_type_value_binding`); Revisit under types-as-values. |
| `quoted_expression` | `let result = 1 + 2` | No quotation; quote syntax does not exist (GRAM-11). | Replace with a real quote fixture once syntax is designed; keep the arithmetic case only if it is not a duplicate. |
| `quoted_expressions_with_complex_nested_structure` | `let template = { let x = 1 + 2 … return y }` | A plain block, with `return` inside an initializer. | Replace (as above). |
| `quoted_label_and_tag` | `:my_label { break :my_label }` | An ordinary label. Nothing is quoted, and there is no tag. | Rename to `labeled_block_with_labeled_break`. |
| `static_labeled_block_expression` | `let x = :labeled { break :labeled 5 }` | Contains no `static`. | Rename (for example `labeled_block_initializer`) to keep the labeled-break case. Add a separate Comptime fixture for a comptime-evaluated binding once `comptime.md` defines how one is expressed (for example, an initializer that calls a comptime intrinsic). |
| `static_let_with_complex_initialization` | `static let config = { … return server + ":" + port }` | Uses `return` inside an initializer and adds a string to an integer. | Rewrite to use `break` with a well-typed value; Comptime. |
| `type_value` | `let x: i64 = 10`, `let ptr: *mut i64`, `let arr: [10]i64` | Type *annotations*, not types used as values. | Rewrite to bind types as values (for example `let t = *mut i64`), or Rename to `type_annotations`. |
| `using_if_shortcut_with_block` | `if (…) { … } else if (interlockable()) :test { break :test 5 } else { … }` | The name is vague. Blank leading and trailing lines are hidden by trimming (TEST-3). | Rename (for example `else_if_chain_with_labeled_block`). |
| `static_block` | `static { let x = 42 let y = "hello" }` | The `name` field is snake_case and the file has a `description` field; other fixtures do not. | Comptime; normalize `name` and `description`. |
| `function_with_static` | `fun compute(x: i32): i32 { static { … } return x }` | Same naming inconsistency. | Comptime; normalize `name` and `description`. |

### Valid fixtures that match but need follow-up

| Fixture | Input (abridged) | Note | Proposed disposition |
|---|---|---|---|
| `static_binding` | `static let z = 3` | Matches. | Comptime |
| `static_function` | `static fun test(): i32 { return 42 }` | Matches. Without `static`, this function calls no intrinsic, so it would be inferred dual-mode. | Comptime: one case where the body calls an intrinsic (comptime-only), and a separate dual-mode case. |
| `static_let_with_type_and_expression` | `static let x: i32 = 1` | Matches. | Comptime |
| `quoted_binding` | ``let result = my.`-path` `` | A backtick identifier; "quoted" collides with code quotation (GRAM-14). | Rename to a `backtick_*` name. |
| `pointer_type` | `let ptr: *mut i64 = 20` | Assigns an integer to a pointer. Valid syntax, invalid semantics. | Revisit; Merge with `mutable_pointer_modifiers`. |
| `function_with_multiple_parameters_of_different_types` | `… : *own i32 { return null }` | `null` is an identifier, not a literal (GRAM-15). | Revisit after GRAM-15 is decided. |
| `own_pointer_modifiers` / `owned_pointer_type`, `raw_pointer_modifiers` / `raw_pointer_type`, `mutable_pointer_modifiers` / `immutable_pointer_type` | `let x: *own i32 = 0` vs `let x: *own i32`, … | Pairs that overlap. | Merge (or keep both, each with a distinct structural assertion). |
| `string_interpolation`, `nested_string_fragments`, `string_with_multiple_interpolations` | `"Hello \(name) …"` | Overlapping interpolation cases. | Merge, or differentiate by assertion. |

### Valid fixtures that match their names (Keep)

`binary_expression`, `binding_with_type`, `binding_without_init`, `block_atom`, `block_atom_with_label`, `block_atom_with_multiple_statements`, `comments`, `complex_nested_expressions`, `complex_operation_with_multiple_operators`, `complex_pointer_types`, `deeply_nested_blocks`, `deeply_nested_if-else_expressions`, `empty_blocks_with_labels`, `function_statement`, `function_statement_with_annotation`, `grouped_expression`, `immutable_pointer_type`, `indexing`, `indexing_with_invocations_and_nesting`, `integer_type`, `invocation`, `lambda_with_multiple_parameters_and_complex_body`, `lambda_with_return_type`, `module_with_multiple_declarations`, `module_with_nested_blocks`, `multi-line_block_comment`, `multiple_annotations`, `multiple_annotations_with_different_arguments`, `multiple_annotations_with_expressions`, `multiple_break_statements_with_different_labels`, `multiple_return_statements_with_different_values`, `multiple_root_statements`, `mutable_binding`, `mutable_let_with_type_and_expression`, `mutable_pointer_modifiers`, `nested_array_types`, `nested_array_types_with_different_dimensions`, `nested_expressions`, `nested_string_fragments`, `nested_string_interpolation`, `owned_pointer_type`, `own_pointer_modifiers`, `pointer_to_array_type`, `raw_pointer_modifiers`, `raw_pointer_type`, `return_with_expression`, `return_with_tag`, `return_with_tag_and_expression`, `simple_binding`, `string_interpolation`, `string_literal`, `string_with_multiple_interpolations`, `trailing_comma_in_annotation_arguments`, `trailing_comma_in_function_call_arguments`, `trailing_comma_in_function_parameters`, `unsized_array_type`.

The "follow-up" table above also lists some of these for merging. `grouped_expression`, `nested_expressions`, and `complex_nested_expressions` currently pass even though PARSE-1 replaces their groups with `Null`. Annotation fixtures pass even though PARSE-8 drops the annotations.

### Error fixtures

| Fixture | Actual input | Problem | Proposed disposition |
|---|---|---|---|
| `incomplete_string` | `let x = \` | Corrupted: has no opening quote. Identical to the next two. | Replace with `let x = "abc`. |
| `incomplete_string_interpolation` | `let x = \` | Identical and corrupted. | Replace with `let x = "a \(b"`. |
| `unterminated_string_interpolation` | `let x = \` | Identical and corrupted. | Replace with `let x = "a \(b + c`. |
| `invalid_array_dimension` | `let x: [\` | Corrupted: has no dimension. | Replace with `let x: [-1]i32` or `let x: [a b]i32`. |
| `mismatched_braces` | `fun test() {` | Unclosed, not mismatched. | Rename to `unclosed_brace`; add a real mismatch (`{ ]`). |
| `mismatched_brackets` | `let x = arr[1` | Unclosed, not mismatched. | Rename to `unclosed_bracket`; add a real mismatch. |
| `mismatched_parentheses` | `let x = (1 + 2` | Unclosed; duplicates the next fixture. | Rename to `unclosed_parenthesis`, or Merge. |
| `missing_closing_parenthesis_in_grouped_expression` | `let x = (1 + (2 * 3)` | Matches. | Keep; Merge candidate with the previous fixture. |
| `invalid_annotation_syntax` | `@(test) fun test() {}` | Matches. | Keep |
| `invalid_break_syntax` | `break :` | Matches. | Keep |
| `invalid_function_call` | `let x = f(,)` | Matches. | Keep |
| `invalid_function_return_type_syntax` | `fun test(): { return 1 }` | Matches. | Keep |
| `invalid_label_syntax` | `: { let x = 1 }` | Matches. | Keep |
| `invalid_let_syntax` | `let = 5` | Matches. | Keep |
| `invalid_module_declaration` | `module 123 {}` | Matches. | Keep |
| `invalid_module_syntax` | `module {}` | Matches; overlaps with the previous fixture. | Keep; consider Merge. |
| `invalid_return_syntax` | `return :` | Matches. | Keep |
| `invalid_type_syntax` | `let x: * {}` | Matches. | Keep |
| `missing_colon_in_parameter` | `fun test(a i32) {}` | Matches. | Keep |
| `missing_expression_after_operator` | `let x = 1 +` | Matches. | Keep |
| `missing_function_body` | `fun test(x: i32): i32` | Matches. | Keep |
| `missing_function_parameter_type` | `fun test(x) {}` | Matches. | Keep |

No error fixture asserts a diagnostic kind or location (TEST-6). TEST-10 lists the missing error coverage. Highest priority: integer overflow (PARSE-6), keyword and type-name boundaries (GRAM-2), unsupported constructs that currently panic (PARSE-3), invalid escapes (GRAM-4), and generic parameter lists, which C-1 now rejects (GRAM-7).
