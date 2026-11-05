# Comptime Reflection Plan — Findings

> Part of the [comptime-reflection plan](./index.md). Every step of the plan cites these IDs. IDs are stable: do not renumber them. Append new findings at the end of a category.

Current-state claims were rechecked against the source, the Pest grammar, manifest/lockfile, test runners, all 100 RON parser fixtures, and the cited current documentation on 2026-11-04. `cargo test --locked --quiet` reported 154 library tests and 2 binary tests passed, with no failures; `cargo fmt --all --check` and `cargo clippy --locked --all-targets --all-features --quiet -- -D warnings` also passed. The passing suite does **not** verify most of the behavior below (see TEST-1).

Behavior marked **(historically probed)** records the earlier CLI probes; those probes were not rerun for this documentation-only revision. Current conclusions are based on the cited code unless an executed check is explicitly named. Line numbers refer to the working tree at review time and may drift. Source citations use repository-relative paths, with `parser.rs` meaning `src/parser.rs`, AST filenames meaning `src/ast/`, and analysis filenames meaning `src/analysis/`.

Some cited documents have since been deleted. Those paths and line numbers are historical evidence, not required inputs; see [Removed documentation](#removed-documentation). Finding IDs and still-relevant compiler requirements are retained.

### Verified implementation boundaries

`src/lib.rs:9–17` and the `src/` module inventory contain only the parser/AST and standalone analysis infrastructure. `src/main.rs:24–40` calls parsing and prints counts. There is no implemented IR, RVSDG region model, lowering, evaluator, borrow checker, reflection/reification pipeline, query database, or documentation generator. Absence claims below refer to that source inventory, not to features described in old documentation.

The AST already owns its children as a Rust tree (`nodes.rs:16–20, 61–77`). It also still carries a `hecs::World` and entity handles (`ast/mod.rs:35–44, 63–118`); the parser only attaches `Position` (`parser.rs:62–70, 154–186`). **C-8 requires removing all ECS use from the AST.** `hecs` is reserved for the IR and semantic analysis, where a distinct component may store each analysis result on its relevant entity. This is an architectural constraint and migration task, not a finding that an owned AST is defective. Existing docs that cite the removed AST-storage finding need their references reconciled under WA-9; this revision edits only the plan.

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
| PARSE-3 | `struct`, `enum`, `interface`, `impl`, and `assignment_stmt` have grammar rules but no statement-dispatch implementation; a pair of any of these kinds reaches `todo!`. Assignment's reachability is complicated by GRAM-12. **(historically probed)** `struct S {}` panics at `parser.rs:206`. | `parser.rs:188–206`; grammar `grammar.pest:182, 191–207` | SYN, CT |
| PARSE-4 | `for` returns a `For` node with no initializer, condition, update, or body. | `parser.rs:411–429`; grammar `grammar.pest:186–188` | SYN |
| PARSE-5 | `case` returns a synthetic `Null` scrutinee and no branches. | `parser.rs:554–567`; see GRAM-10 | SYN, ASTIR |
| PARSE-6 | Numeric conversion uses `parse().unwrap()` and panics when an integer is out of range (for example `9223372036854775808`) or when a span contains whitespace (see GRAM-1). | `parser.rs:869` (integer), `parser.rs:875` (float) | SYN, ASTIR |
| PARSE-7 | Defensive `todo!` arms remain in expression, atom, and atomic-value dispatch. They must become internal-error diagnostics or be proven unreachable. | `parser.rs:476, 855, 886` | SYN |
| PARSE-8 | `parse_decorated_statement` collects direct comment pairs and parsed annotations into local vectors and discards both; only the statement is returned. Implicit comment matching can also place comments outside that collector. The parser never attaches `Comment` or `Annotation`, retains no comment depth, and exposes no documentation extraction path. | `parser.rs:110–150`; `grammar.pest:6–8, 243`; `components.rs:52–64` | SYN, ASTIR, CT |
| PARSE-9 | The parser's general pattern on unexpected shapes is to invent values (`Null`, `_`) and report success, instead of returning a positioned error. This violates constraint C-5. | PARSE-1, PARSE-2, PARSE-5 | SYN |
| PARSE-10 | `if_branch` permits an absent condition, but `IfBranch::parse` unconditionally unwraps it. The accepted grammar shape `if {}` therefore has no faithful conversion. Either represent an approved meaning or reject it with a positioned diagnostic. | `grammar.pest:150`; `parser.rs:524–549` | SYN |

## GRAM — Grammar (`src/grammar.pest`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| GRAM-1 | Identifier, number, and string rules are non-atomic, so implicit `WHITESPACE`/`COMMENT` skipping happens *inside* tokens. **(historically probed)** `let x = ab cd` is accepted as a single identifier. | `grammar.pest:35–53` | SYN |
| GRAM-2 | Keywords and type names have no token boundary at their use sites. With PEG ordered choice, this either merges tokens or rejects valid identifiers. **(historically probed)** `letx = 1` parses as `let x = 1`, and `let x: *mutable` parses as `*mut able`. `let x = true_value` and `let x = i32_max` are rejected because `true` or `i32` matches first. Because `atomic_type` is tried before `binding`, identifiers such as `f1`, `i8`, and `u2` are classified as types. | keywords `grammar.pest:11–32`; `ident` 39; `integer_type`/`floating_type` 116–117; `pointer_type` 119; `atom` order 135 | SYN |
| GRAM-3 | `atomic_value` tries `integer` before `float`; `3.14` can instead become an operation with integer operands and `.` as an operator. **(historically probed)** `let x = 3.14` produces more AST entities than `let x = 3`; entity counts alone do not establish the resulting expression kind. The inline module test contains `3.14` but never asserts a float node. | `grammar.pest:99, 132, 148`; `parser.rs:635–656, 1083–1095` | SYN |
| GRAM-4 | Strings have no escape sequences. `string_fragment` excludes `\`, and `\(` starts interpolation. **(historically probed)** `let x = "a\nb"` is rejected. | `grammar.pest:50–53` | SYN |
| GRAM-5 | `label` and `tag` are non-atomic and accept bare `ident_char+`, so labels such as `:1` are accepted and whitespace can be skipped inside them. | `grammar.pest:41–42` | SYN |
| GRAM-6 | Dead alias rules (`*_statement`, `call`, `index`, `value`, `case_arm`) and an unused `comment` wrapper. Of the compatibility/postfix alias group, only `field` is used by the active grammar and conversion. | `grammar.pest:138–141, 214, 217–241`; `parser.rs:811` | SYN |
| GRAM-7 | Generic parameter lists `<T, ...>` on `struct`, `enum`, `interface`, and `impl` contradict constraint C-1 (no generics). | `grammar.pest:191–207` | CT, SYN |
| GRAM-8 | The legacy `static_kw` forms (`static let`, `static fun`, `static { }`) are retired with no replacement keyword or annotation (C-2). Comptime-ness is inferred from calls to built-in comptime intrinsics (META-11), so no grammar marks comptime code. General annotation syntax is unaffected. | legacy forms `grammar.pest:22, 32, 170, 178, 180` | CT, SYN |
| GRAM-9 | `for … in` uses a literal `"in"`, which is not a keyword and has no boundary. | `grammar.pest:187` | SYN |
| GRAM-10 | The `case` grammar has no scrutinee (`case_branch = case_kw ~ expression ~ expression`), but the AST requires `Case.value` plus pattern and guard branches. | `grammar.pest:151–152`; `nodes.rs:253` (`Case`), `261` (`CaseBranch`) | SYN, ASTIR |
| GRAM-11 | There are no dedicated quotation or splice grammar rules or AST nodes. `~` is accepted as both a unary and general operator, not as a splice. Required syntax is now fixed: **backticks quote code, `~` splices, `@symbol` / `@symbol(args)` annotates**. Replace generic treatment at splice sites with a dedicated parse/AST representation; define binding, nesting, and valid splice contexts. `@` is not quotation syntax and the historical `$()` proposal is not adopted. Reflection intrinsics do not replace the required quote/splice surface forms. | `grammar.pest:81, 146, 166, 215`; `nodes.rs:196–209`; C-3 | SYN, REFL |
| GRAM-12 | `assignment_stmt = expression "=" expression` is ambiguous with `=` as a binary operator. In practice `x = 1` is parsed as an operation. | `grammar.pest:182`; `operator` `grammar.pest:102` | SYN, ASTIR |
| GRAM-13 | A `;` separator in a block must be followed by another statement, so `{ let x = 1; }` is rejected. The root rule does not accept `;` at all. | `grammar.pest:246–249` | SYN |
| GRAM-14 | Backticks currently delimit identifiers via `ident_backtick`, including ``my.`-path` ``. This conflicts with their required role as code quotation, not merely fixture naming. Redesign/remove that identifier rule and migrate `binding`/`quoted_binding`; if escaped identifiers remain, their syntax must be explicitly unambiguous with quotation. Define backtick boundaries, escaping, nested quotation, and quote/splice interactions before changing the grammar. | `grammar.pest:38–39`; `test_cases/parser/valid/binding.ron:4`, `quoted_binding.ron:4`; C-3 | SYN, REFL, TST |
| GRAM-15 | The AST has `Literal::Null`, but the grammar has no `null` literal. `return null` parses `null` as an identifier. | `nodes.rs:382` (`NullLiteral`); fixture `function_with_multiple_parameters_of_different_types` | SYN, ASTIR |
| GRAM-16 | `COMMENT` participates in Pest's implicit matching, while `decorated_statement` also matches comment rules explicitly. The silent wrapper does not itself produce a pair; the named child rules can, but the conversion has no lossless collection/attachment model (PARSE-8). There is no distinct doc-comment grammar. Preserve all comments in AST-owned data, including trivia between tokens, before/after declarations, at EOF, and in comment-only files; choose documentation classification and attachment rules rather than assuming `is_doc_comment` is populated. | `grammar.pest:6–8, 214–215, 243–249`; `parser.rs:100–132`; `components.rs:52–57` | SYN, ASTIR |
| GRAM-17 | Nested block comments are already expressed recursively, and `multi-line_block_comment.ron` contains nesting. However, no parser/AST field tracks comment depth and no test asserts it. Preserve that language feature while adding explicit depth accounting and retained nesting: increment at each opener, decrement at each closer, finish only at zero, reject unterminated nesting, and define a safe excessive-depth diagnostic. Evaluate compound-atomic/comment rules or explicit trivia parsing so implicit skipping cannot corrupt comment contents or hide delimiter boundaries. Keep string/interpolation lexical contexts distinct from comment contents. | `grammar.pest:6–8, 50–53`; `parser.rs:37–45, 110–132`; `components.rs:52–57`; `test_cases/parser/valid/multi-line_block_comment.ron:4–7`; C-10 | SYN, ASTIR, TST |
| GRAM-18 | Current pointer types use `*own`, `*raw`, `*mut`, and bare `*`, overlapping the multiplication token and unary `*` grammar. The required compiler syntax changes the four pointer-type prefixes to `&own`, `&raw`, `&mut`, and bare `&`; `*` remains multiplication, not pointer syntax. Migrate grammar, type examples, fixtures, parser/printer round trips, and diagnostics without changing multiplication behavior. | `grammar.pest:81, 102, 119, 146`; `types.rs:38–42, 67–74`; C-3 | SYN, ASTIR, SEM, TST |

## AST — Syntax tree (`src/ast/`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| AST-1 | The unused `Annotation` component stores `Vec<String>`, whereas conversion temporarily builds `Vec<Expression>` and discards it. There is no annotation field on a statement. Preserve annotation names, expression arguments, order, target, and spans in ordinary AST data, not ECS components. | `components.rs:59–64`; `parser.rs:135–150`; `nodes.rs:28–41`; C-8 | ASTIR, CT |
| AST-2 | `Statement::Static`, the `Static` node, and the `is_static` fields encode the legacy keyword forms. Under C-2 they are removed, not renamed. Comptime/runtime mode becomes a compiler-derived, per-item fact (META-11), not an AST node or flag. | `nodes.rs:40, 55, 76, 101, 188–190` | ASTIR, CT |
| AST-3 | `Operation.elements` is a flat, source-ordered operand/operator list. Precedence is never resolved, and no design says where it is resolved or whether operators are comptime-extensible. | `nodes.rs:283–295` | ASTIR, CT |
| AST-4 | Integer literals are stored as `i64`. `u64` values above `i64::MAX` cannot be represented, and the literal's spelling is lost. | `nodes.rs:347–349` | ASTIR |
| AST-5 | The AST has no representation for code quotation or splicing. Add dedicated non-ECS representations for the required backtick and `~` syntax, with nested-code structure and provenance; define their lowering to comptime code values alongside reflection/reification values. Ordinary intrinsic calls are not a replacement for these nodes. No comptime-mode marker node is wanted (C-2). | `nodes.rs:196–209`; GRAM-11, META-2, META-11 | ASTIR, REFL |
| AST-6 | Two stage types exist: the ECS `Stage` component and `analysis::Stage`. Both are replaced by the concepts that ANL-10 defines. | `components.rs:33–35`; `analysis/stage.rs` | ASTIR, CT |
| AST-7 | AST entity handles depend on allocation order and are copied by node `Clone`; they do not establish stable source identity across parses, even when numeric values happen to repeat. Remove them from the AST under C-8 and introduce stable compiler IDs independently of the future IR's allocation-local ECS handles. | `ast/mod.rs:64–82`; `nodes.rs:16–20, 69–77` | INCR, ASTIR |
| AST-8 | There is no index of item or module ownership. A module is a name plus inline statements. | `nodes.rs:62` (`Module`) | INCR, ARCH |
| AST-10 | `Comment { text, is_doc_comment }` is only an unused component type. The owned AST has no comment records, nesting/depth, source attachment, or documentation traversal. Introduce ordinary AST-owned comment data with kind, raw text, ranges, ordering, depth/nesting, and attachment, preserving unattached trivia as well as documentation comments. This must support extraction without reparsing discarded source and printing without losing nested comments. | `components.rs:52–57`; `nodes.rs:16–20, 28–41, 176–190`; `parser.rs:110–132`; `printer.rs:68–97`; C-8, C-10 | ASTIR, SYN |

## PRINT — Source printer (`src/ast/printer.rs`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| PRINT-1 | The printer emits a different surface language: `fn` instead of `fun`, trailing semicolons, `\(…) => …` lambdas, apostrophe labels, `while` without parentheses, `${}` interpolation (which re-parses as plain text), and string escapes the grammar rejects. | `printer.rs:110–164, 195–217, 432–450, 511–532` | SYN, TST |
| PRINT-2 | Printer tests assert this incompatible output. | `printer_tests.rs:161, 214, 286`; `printer_additional_tests.rs:33, 57, 123` | TST |
| PRINT-3 | The printer hand-emits strings independently of Pest; it neither uses parsed spans nor preserves comments/annotations. Pest can recover matched source text, and current `Pair::Display` behavior differs from the locked release, but neither provides an unparser for a changed/generated AST. Prefer source replay for unchanged syntax plus a grammar-aligned emitter for changed/generated nodes, using owned source/ranges and retained trivia rather than long-lived `Pair` borrows in the AST. Step 1 must compare that approach with a lightweight lossless token/CST representation and document the tradeoff. | `printer.rs:13–16, 68–97, 110–165, 511–555`; `parser.rs:58–77`; [Pest printing investigation](#pest-printing-investigation); DEP-1 | SYN, ASTIR, TST |

### Pest printing investigation

Checked against the published sources on 2026-11-04:

| API | What it actually provides |
|---|---|
| [`Pair::as_str()` / `as_span()`](https://docs.rs/pest/2.9.5/pest/iterators/struct.Pair.html) | A slice/span of the original input for that matched rule, not regenerated source. Available in the locked 2.8.0 release too. |
| [`Pair::Display` in 2.9.5](https://docs.rs/pest/2.9.5/src/pest/iterators/pair.rs.html#341-368) | Default formatting / `to_string()` writes `as_str()`; alternate formatting (`{pair:#}`) writes the rule/span tree. **Yes, this release can stringify a parsed pair as its source.** |
| [`Pair::Display` in locked 2.8.0](https://docs.rs/pest/2.8.0/src/pest/iterators/pair.rs.html#341-364) | Writes a rule/span tree, not Patina source. Do not apply the new behavior to the current dependency without upgrading. |
| [`Pairs::as_str()` / `get_input()` / `concat()` / `Display`](https://docs.rs/pest/2.9.5/src/pest/iterators/pairs.rs.html) | `as_str()` spans the remaining pairs, including gaps; `get_input()` returns the original input; `concat()` joins individual matched slices and can lose gaps; `Display` formats a bracketed list. None is a canonical AST printer. |

The recommended split is **lossless replay for unchanged source** (retain the complete source and ranges, including boundary trivia) and **grammar-aligned emission for changed or reified AST nodes**. A source slice cannot synthesize code that never had a source slice. Both paths must preserve comments and annotation ordering, use the required quote/splice syntax, and be checked by Pest parsing plus structural AST assertions; replay additionally needs exact byte equality. Do not concatenate nested pair strings, which would duplicate overlapping text. Recheck these APIs after selecting dependency versions; no printer library or CST dependency is chosen by this plan update.

## VIS — Traversal (`src/ast/visitor.rs`, `src/ast/mod.rs`)

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| VIS-1 | `ComponentFilterWrapper` does not forward `visit_expression` or `visit_literal`, so wrapped visitors silently lose those callbacks even when the filter always passes. | trait methods `visitor.rs:21, 28`; wrapper `visitor.rs:357, 372–544` | ASTIR |
| VIS-2 | `Traversable<C>` and `GenericVisitor<C>` have no implementations for any AST node. The default `accept` does not recurse. | `ast/mod.rs:120–136`; `visitor.rs:310–317` | ASTIR |
| VIS-3 | Traversal is preorder with unconditional recursion, has no exit/pruning hooks, and receives only `&Ast`. A visitor can accumulate external facts, so analysis is not categorically impossible; however, this API has no balanced scope-exit event or direct AST-mutation access. Design scoped traversal and query-owned facts rather than relying on ECS fact attachment. | `visitor.rs:15–52, 85–105, 274–280` | ASTIR, ARCH |

## POS — Source provenance

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| POS-1 | The root position receives `Some(filename)`; ordinary node conversion uses the free helper with `filename: None`. The filename-aware `ParserVisitor::add_position` method exists but conversion does not call it. | `parser.rs:62–70, 154–186` | SYN, INCR |
| POS-2 | Call, index, and field nodes have no position. | `parser.rs:754–825` | SYN |
| POS-3 | The CLI calls `parse_file` instead of `parse_file_with_name`, even though it knows the filename. | `main.rs:14–24` | ARCH |
| POS-4 | Diagnostic rendering ignores `Position.filename`. | `analysis/diagnostics.rs:170–200` | ARCH |
| POS-5 | There is no `FileId`, revision-aware span, or generated-code provenance (expansion site plus definition site). There are no semantic fingerprints yet; the future design must separate location data from them. | `components.rs:9–29`; [implementation boundaries](#verified-implementation-boundaries) | INCR, REFL |
| POS-6 | `Position` already describes a **byte range** with `offset` and `length`, but exposes only the start `line`/`column`, not the end. The unused custom `ParseError` has only a start point; actual grammar errors are Pest errors propagated by `?`. Analysis diagnostic rendering prints only the start. Provide a unified source range with start and end line/column for multi-line underlining. Decide end exclusivity, column units, byte offsets alongside line/column, zero-width/multi-line spans, synthetic and generated provenance (POS-5), conversion from Pest's span endpoints, rendering, file identity, and exclusion from semantic fingerprints. Store AST ranges in ordinary node data, not ECS (C-8). | `components.rs:9–29`; `parser.rs:17–27, 49–70, 154–186`; `analysis/diagnostics.rs:170–200`; `analysis/symbol_table.rs:33` | SYN, ASTIR, ARCH, INCR |

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
| ANL-9 | `Arena<T>` is an append-only `Vec<T>` returning insertion-order `Id<T>` indices. Its production users are `SymbolTable` (scopes/symbols) and `TypeRegistry` (types), neither connected to parser output. It has no removal/generation validation and no stable cross-revision identity. The separate tag aliases cause ANL-1. Plan to replace the incompatible ID families and storage APIs with stable query identities, allocation-local `hecs` entities **for the IR**, and a type interner if needed. The future ECS IR is not evidence that symbol/type tables must themselves be ECS; their ownership belongs in the design. | `arena.rs:3–49`; `id.rs:4–56`; `symbol_table.rs:7–8, 81, 150, 181`; `types.rs:17, 185–196` | ASTIR, INCR |
| ANL-10 | "Stage" conflates three concepts and needs to be rethought and renamed. `analysis::Stage(u32)` hard-codes a numbered level, with `COMPILE_TIME = 0` and `RUNTIME = 1`. The ECS `Stage` component duplicates it (AST-6), and the symbol table, analysis context, and type registry thread a "current stage" through lookup and types. Under C-2 and Goal 4 the plan needs three separate concepts: (1) **mode**, whether a function runs at comptime, runtime, or both, which is an inferred per-function fact (META-11); (2) **sealing**, whether an item is fully reified, so that its facts can no longer change and later comptime code may reflect it (META-9); and (3) **reification order**, a partial order derived from the reflects-on and reifies edges between items, in which an item's comptime code runs only after the items it reflects are sealed, and cycles are diagnosed. A global integer level is at most a display artifact of that order. The name is also ambiguous: "stage" collides with compiler pipeline stages (parse, resolve, lower), and "phase" collides with the removed phase-separation plan. `comptime.md` must separate the concepts and name each one. Candidates for (3) include *round*, *layer*, *tier*, *level*, *generation*, and *epoch*, or no name at all if only the order matters. Record the rejected names and their collisions. Until then, "stage" in this plan is a provisional legacy term. `Stage`, `declare_staged`, `lookup_at_stage`, `lookup_all_visible_stages`, `Type::Code { from_stage }`, `StagedType`, and the stage scopes are to be replaced or removed accordingly. | `analysis/stage.rs:3–54`; `components.rs:33–35`; `analysis/symbol_table.rs:223–276`; `analysis/context.rs:18, 55, 70`; `analysis/types.rs:60, 94, 313–353` | CT, IREFL, INCR, ASTIR |
| ANL-11 | No approved semantic-analysis design exists. Required work includes resolution/type checking, the borrow checker (ANL-12), mode inference (META-11), comptime-computed types interleaved with evaluation, generated-code typing and hygiene, effects through RVSDG state edges, reflection/reification ordering, and incremental invalidation. Dead-code elimination is not required for correctness. These are requirements, not a measured claim that the present compiler analyzes more than Rust. `src/analysis/` performs none of these on parsed programs. Specify each query's inputs/outputs, stable key, fixed-point strategy, ordering/interleaving, reflection exposure, diagnostics, and performance budget. | `src/analysis/mod.rs:1–19`; `types.rs:313–355`; `src/main.rs:24–40`; ANL-4, C-6, C-9 | SEM, ASTIR, INCR |
| ANL-12 | **No Patina borrow checker exists.** `PointerModifier` enumerates four kinds but performs no move, loan, alias, lifetime, or drop checks. Design and implement an incremental checker over resolved places and RVSDG value/state flow: moves and partial moves, initialization/use-after-move, shared/exclusive loans and reborrows, writes and drops while borrowed, escapes and return lifetimes, calls/closures, branches/loops, and conversions among `&own`, `&raw`, `&mut`, and `&`. Define raw-pointer guarantees and obligations explicitly instead of assuming Rust's semantics. Apply checks to comptime, runtime, and reified code before execution, including preventing comptime pointers from escaping. Lifetime constraints are not the same thing as RVSDG regions. | `types.rs:38–42, 67–74, 313–355`; `src/analysis/mod.rs:1–19`; C-9 | SEM, ASTIR, INCR, TST |

## IR — Intermediate representation and driver

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| IR-1 | There is no IR, lowering, or backend in the source. C-8 requires a new RVSDG represented with `hecs` entities/components, separate from the AST. Semantic-analysis facts may also be represented as distinct components on relevant analysis entities. Cover resolved references, runtime versus type/meta values, operations, nested regions/ports, value/state dependencies, boundaries between reification-ordered items, validation, and stable identities. Do not describe the current AST world as an existing IR. | `src/lib.rs:9–17`; `src/main.rs:24–40`; [implementation boundaries](#verified-implementation-boundaries); C-8 | ASTIR, INCR |
| IR-2 | The executable is a single-file parse demo. It prints entity and statement counts and exits, with no compiler session or pipeline. | `main.rs:14–40` | ARCH |
| IR-3 | **RVSDG regions are not implemented.** There are no graph-region entities/components, region owners, arguments/results, typed ports, cross-region routing, or region validators. AST `Block` and analysis `Scope` are not RVSDG regions. Implement structural nodes for function bodies, alternatives and loops, with explicit boundary edges, value/state arity/type checks, ownership/scoping invariants, and source provenance before relying on region dataflow in evaluation or borrow checking. | `nodes.rs:176–181`; `symbol_table.rs:14–27`; `src/lib.rs:9–17`; IR-1 | ASTIR, SEM, INCR, TST |

## INC — Incremental compilation

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| INC-1 | There is no compiler database or session. Every parse builds a fresh AST and its current position side table; nothing reuses unchanged parsed files. | `parser.rs:54–59`; `ast/mod.rs:63–76` | INCR, ARCH |
| INC-2 | Identities depend on allocation. `Arena::insert` uses the current vector length, and ECS entities are world-local. | `arena.rs:13–17`; `ast/mod.rs:64–82` | INCR |
| INC-3 | There are no fingerprints, recorded dependencies, or revisions. | `parser.rs:54–77`; `analysis/context.rs:5–24`; [implementation boundaries](#verified-implementation-boundaries) | INCR |
| INC-4 | There is no module or import graph, and no file-to-module ownership. | `nodes.rs:61–66`; `analysis/symbol_table.rs:14–27`; `src/main.rs:14–40` | INCR, ARCH |
| INC-5 | The only cache is a per-registry structural type interner. It is not incremental. | `types.rs:185–208` | INCR |
| INC-6 | Planned comptime file I/O has no tracked-input contract. | `docs/plans/phase-separation/step-4.md:35–41` | INCR, REFL |
| INC-7 | Diagnostics are not produced per query and cannot be replaced or remapped per revision. | ANL-8 | INCR |
| INC-8 | The removed performance and AST visitor RFCs deferred incremental and query architecture until late optimization phases. Incremental foundations must instead precede semantic analysis; removing those RFCs does not implement that foundation. | `docs/rfcs/performance.md:142–145`; removed AST visitor RFC, Implementation Plan | INCR |

## META — Comptime metaprogramming and reflection

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| META-1 | There is no comptime evaluator or interpreter. | `src/lib.rs:9–17`; `src/main.rs:24–40`; [implementation boundaries](#verified-implementation-boundaries) | CT, ASTIR |
| META-2 | There are no executable reflection values or intrinsics exposing compiler facts. `TypeRegistry` IDs and `Type::Code` are standalone analysis data, not values available to Patina comptime code. | `types.rs:15–64, 313–355`; META-1 | REFL |
| META-3 | There is no reification and no loop that reanalyzes generated code. | `parser.rs:54–77`; `src/main.rs:24–40`; META-1 | REFL, ARCH |
| META-4 | The removed staging RFC deferred hygiene to "advanced features"; it is needed from the first generated binding and remains unimplemented. | `docs/rfcs/staging-features.md:161–165` | REFL |
| META-5 | There is no identity scheme for generated items. Stable identity is needed for incremental reuse, diagnostics, and removal of stale items. | `ast/mod.rs:63–82`; `analysis/id.rs:4–56`; META-3 | REFL, INCR, IREFL |
| META-6 | Generics are planned in docs and grammar. Under C-1 they must be replaced by comptime functions that take and return types, including for parameterized `struct`/`enum`/`interface`/`impl`. | GRAM-7, ANL-6, DOC-8 | CT |
| META-7 | The legacy `static` markers in fixtures, the AST, and the docs must be retired without replacement syntax (C-2). Fixtures that exercised `static` must be rewritten so that comptime evaluation follows from calls to comptime intrinsics (META-11), or replaced. | GRAM-8, AST-2; fixtures `static_*`, `function_with_static` | CT |
| META-8 | Annotations are parsed but erased before semantic processing (PARSE-8). No contract defines what an annotation means or how it attaches to its target. Annotations carry no comptime/runtime meaning (C-2), but comptime code may need to reflect on them. | PARSE-8, AST-1 | CT, REFL |
| META-9 | There is no model of item sealing ("fully reified"), no reification order that guarantees comptime reflection sees only sealed items (formerly "stage *n + 1* sees completed stage *n*"), and no invalidation along that order. ANL-10 separates these concepts from mode and renames "stage". | `analysis/stage.rs:3–54`; `analysis/context.rs:5–24`; META-3, INC-3 | IREFL |
| META-11 | There is no call graph and no comptime/runtime mode inference (C-2). The design must define: the set of built-in comptime intrinsics; propagation from a function that calls an intrinsic to its transitive callers; dual-mode functions; recursion and call-graph cycles; indirect calls through function values and through reified code; what triggers comptime evaluation when nothing marks it (for example, a type position, or a value that a comptime-only call requires); which comptime results can be lifted into runtime code, and how; the diagnostic when runtime code would call a comptime-only function; how the bindings and blocks formerly written `static let` and `static { }` are expressed; and how modes relate to reification order (ANL-10). Mode must be an incremental per-item fact that is invalidated only when a callee's mode actually changes. | `stage.rs:49–53`; ANL-3, GRAM-8 | CT, INCR, IREFL |
| META-12 | Dead-code elimination is not required for correctness. If implemented as an optional optimization, its roots/reachability rules may be incremental, but optimized and unoptimized builds must have the same observable semantics. Deferring or omitting DCE is valid and must not block a correct compiler pipeline. | IR-1 | CT, ASTIR, INCR, TST |

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
| TEST-9 | `insta` is not a dependency and no snapshot files are tracked. The testing guide now says so, but the status page and testing RFC still claim snapshot coverage. Whether to adopt snapshots remains a design question. | `Cargo.toml:14–23`; `docs/developer/testing.md:27`; `docs/status/current.md:74`; `docs/rfcs/testing.md:5`; DOC-5 | TST |
| TEST-10 | The RON fixtures have no dedicated float/escape, loop, case, aggregate-declaration, assignment-statement, boundary, overflow, or source-position coverage. Bindings contain `=`, but that is not assignment-statement coverage. An inline module test contains `3.14` without asserting a float AST (GRAM-3). The nested-comment fixture proves only acceptance, not depth or retention. Add structural tests for comments at every trivia position, exact depth/nesting and unterminated nesting, documentation extraction, backtick quotations/`~` splices versus annotations, `&` pointer prefixes versus multiplication, filename-bearing diagnostics, and printer round trips; semantic tests for borrow checking across pointer kinds/modes, RVSDG regions, comptime inference/evaluation, generated code, and incremental equivalence/reuse. DCE tests are optional optimization-equivalence tests, not correctness gates. | `test_cases/parser/`; `src/parser/tests.rs:18–77`; `src/parser.rs:1083–1095`; `test_cases/parser/valid/multi-line_block_comment.ron:4–7` | TST |
| TEST-11 | Historical lint failure: three AST accessor return types hid elided lifetimes. The current signatures spell `'_`, and the local formatting and CI-flag clippy commands passed on this revision. This is not a claim about a current remote workflow run. Keep these checks passing in implementation steps and recheck after dependency upgrades. | `src/ast/mod.rs:92–110`; `.github/workflows/ci.yaml:25–28`; validation recorded above | **Resolved** locally; TST maintains the regression gate |

## DOC — Documentation

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| DOC-1 | The status page still claims a complete parser, full AST generation, error recovery, positions for all nodes, snapshot verification, and 200+ tests, despite its warning banner. It also implies future generics and complete stage infrastructure. The AST-tree description was corrected, but stale finding references remain. Treat timing/performance claims as unmeasured, not a verified baseline. | `docs/status/current.md:3, 11–16, 67–96, 110–116`; PARSE-*, TEST-9 | all; WA-9 |
| DOC-2 | The stage-levels design described implemented stage inference and used `@[stage(n)]`, `$()`, `fn`, generics, and stage polymorphism. | `docs/designs/stage-levels.md:5, 75–81, 135, 194` | **Resolved** by deleting the superseded design; the replacement design remains a Step 1 task |
| DOC-3 | The visitor-pattern doc's nonexistent `SyntaxNode`/walk API was replaced with the actual API. It still overstates VIS-3 by saying scoped resolution and fact storage cannot be built on the visitor at all; visitors can accumulate external facts, though balanced exits are missing. | `docs/developer/architecture/visitor-pattern.md:103–134`; `src/ast/visitor.rs:15–52, 85–105` | ASTIR, ARCH; WA-9; **partially resolved**, correct the remaining limitation claim |
| DOC-4 | The performance RFC quoted analysis throughput numbers with no pipeline or benchmark behind them. | `docs/rfcs/performance.md:77–80` | **Resolved** by deleting the invalid RFC |
| DOC-5 | The testing guide documented `insta` workflows that do not exist. | Former `docs/developer/testing.md:9, 49–61, 79, 141` | **Resolved** by recreating the guide from the current tests and fixtures; whether to adopt snapshot testing remains TEST-9 |
| DOC-6 | The old getting-started examples used unsupported syntax. The replacement fixes those examples, but still lists float literals and `continue` as meaningfully supported. `continue` has no keyword or statement node; it is only an identifier. The float issue remains GRAM-3. Its dependency list gives manifest requirements, not locked or current release versions; update that list with the future upgrades (DEP-1). | `docs/developer/getting-started.md:9, 84–88`; `grammar.pest:11–32, 132, 212`; `nodes.rs:28–41`; DEP-1 | SYN, ARCH; WA-9; **partially resolved**, remaining claims need correction |
| DOC-7 | The AST guide must describe ECS as excluded from the AST target architecture, not as an optional side table. It may document the existing `Ast::world`/entity fields only as legacy implementation to remove; filename/source identity must remain available for diagnostics. Semantic-analysis components belong to relevant analysis entities, not AST nodes. | `docs/developer/ast.md`; `ast/mod.rs:35–44, 63–118`; `components.rs:9–29`; C-8, POS-6 | ASTIR, ARCH; WA-9 |
| DOC-8 | The old language proposals specified generics, contradicting C-1, and the developer docs did not distinguish Rust implementation generics from forbidden Patina language generics. | Removed: `docs/rfcs/type-system.md`, `staging-features.md`, `codebase.md`, `performance.md`; `docs/designs/stage-levels.md`; `docs/plans/phase-separation/step-5.md`. Current: `overview.md:55, 63`; `ast.md:23`; `visitor-pattern.md:124` | **Resolved**: the proposals were removed, and the developer docs now identify `Ast<C>` and `GenericVisitor<C>` as Rust marker parameters and `Type::TypeParameter`/`Type::Generic` as conflicting with C-1 (ANL-6). The status page's generics limitation remains under DOC-1 |
| DOC-9 | `docs/README.md` links to nonexistent `user/`, `user/getting-started.md`, `status/component-based-ast-implementation.md`, and `status/IMPLEMENTATION_COMPLETE.md`. | `docs/README.md` | **Resolved** by the plan-setup change |
| DOC-10 | Earlier `AGENTS.md` claims about completed parsing/analysis, snapshot tooling, and nonexistent APIs were corrected. It still cites the removed AST-storage finding and describes legacy stages provisionally. Update those references and distinguish required ownership semantics from the current unchecked pointer syntax. | `AGENTS.md:48–55, 61–65, 82–103`; `grammar.pest:119`; ANL-4, ANL-10, ANL-12 | ARCH, ASTIR; WA-9; historical corrections retained, current references remain |
| DOC-11 | Phase-separation step 3 required duplicated per-stage ASTs, which conflicts with item-level queries. | `docs/plans/phase-separation/step-3.md:11–18` | **Resolved** by deleting the superseded plan; the query-based replacement remains in the carry-over table |
| DOC-12 | `docs/rfcs/codebase.md` was not linked from the docs index. | `docs/README.md` | **Resolved** by the plan-setup change; the RFC and its index entry have since been removed |
| DOC-13 | The codebase RFC linked to a nonexistent `../development/getting-started.md`; the guide is at `../developer/getting-started.md`. | `docs/rfcs/codebase.md:137` | **Resolved** by deleting the invalid RFC |
| DOC-14 | The main Markdown architecture descriptions now identify the owned tree and planned ECS IR, and the old summary is gone. However, Rust module docs still call the AST component-based/ECS, and several Markdown files cite the removed storage finding or describe ECS removal as optional. Those claims must be reconciled with the mandatory non-ECS AST in C-8. | `src/lib.rs:1–7`; `src/ast/mod.rs:1–6`; `docs/README.md:5–10`; `docs/developer/architecture/overview.md:46–50`; `docs/developer/architecture/visitor-pattern.md:134` | ARCH, ASTIR; WA-9; **partially resolved** |

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
| `AST_IMPLEMENTATION_SUMMARY.md` (repository root) | Presented the AST as an ECS structure with generic visitors. | C-8, VIS-2, DOC-14 |

The remaining RFCs are inputs for problem discovery, not approval of their sample APIs or implementation order. Step 1 must reconcile them with the active constraints before they inform implementation plans.

## DEP — Dependencies

Manifest requirements are semver ranges, **not** the versions actually in use. The lockfile and crates.io registry were checked on 2026-11-04; the releases below are candidates to assess during implementation, not an approved upgrade or a promise to use stale version pins later.

| Direct dependency | `Cargo.toml` requirement | `Cargo.lock` version | Latest non-yanked stable observed | Reported minimum Rust version for candidate |
|---|---|---|---|---|
| `hecs` | `0.10.5` | `0.10.5` | `0.11.2` | `1.81` |
| `pest` | `2.7` | `2.8.0` | `2.9.5` | `1.83` |
| `pest_derive` | `2.7` | `2.8.0` | `2.9.5` | `1.83` |
| `ron` | `0.8` | `0.8.1` | `0.12.3` | `1.64.0` |
| `serde` | `1.0` (derive) | `1.0.219` | `1.0.229` | `1.56` |
| `thiserror` | `1.0` | `1.0.69` (direct); `2.0.12` also locked transitively | `2.0.21` | `1.77` |
| `tempfile` (dev) | `3.8` | `3.20.0` | `3.27.0` | `1.63` |

Sources: `Cargo.toml:1–23`, package entries in `Cargo.lock`, and registry metadata for [hecs](https://crates.io/api/v1/crates/hecs), [pest](https://crates.io/api/v1/crates/pest), [pest_derive](https://crates.io/api/v1/crates/pest_derive), [ron](https://crates.io/api/v1/crates/ron), [serde](https://crates.io/api/v1/crates/serde), [thiserror](https://crates.io/api/v1/crates/thiserror), and [tempfile](https://crates.io/api/v1/crates/tempfile). Reported MSRVs are package metadata; validate the whole resolved graph on the chosen project toolchain.

The replacement decision is explicit: remove Patina's direct `thiserror` dependency and use `derive_more` error/display derives. The version, feature set, MSRV, and generated error API for `derive_more` must be checked during dependency implementation. A transitive `thiserror` is acceptable only if another selected dependency still requires it.

| ID | Finding | Evidence | Owner |
|---|---|---|---|
| DEP-1 | Direct dependencies need a reviewed modernization rather than blindly pinning the newest releases. `hecs` 0.11 and `ron` 0.12 require changes beyond current manifest ranges; Pest 2.9, serde 1.x, and tempfile 3.x fit broad current ranges but are not selected by the current lockfile. Plan manifest/lockfile updates, API/fixture migrations, MSRV policy, compatible Pest derive/generator/meta versions, and the required replacement of direct `thiserror` with `derive_more`. Validate the selected derive_more version/features and generated error API. Rerun behavior-focused parser/printer tests, not just compilation; specifically account for the `Pair::Display` change (PRINT-3). | version table above; `Cargo.lock` Pest stack entries at `2.8.0`; `.github/workflows/ci.yaml:14–28` | ARCH, SYN, TST |
| DEP-2 | `thiserror` is declared directly but unused in `src/`; Patina error types currently implement `Display` and `std::error::Error` manually. Replace the direct dependency with `derive_more` and use its derives for error/display implementations where appropriate. Audit error API and feature/MSRV compatibility; remove the direct `thiserror` requirement from the manifest and lockfile, while allowing a transitive copy only if another dependency requires it. Audit other direct dependencies for actual use before upgrading, without forcing AST data into ECS. | `Cargo.toml:20`; `parser.rs:23–29`; `analysis/symbol_table.rs:304–317`; `Cargo.lock` `patina`/`pest`/`thiserror` entries; source search for `thiserror` | ARCH |

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
| `binding` | ``my.`-path` `` | A path-expression statement, not a binding; also exercises the backtick-identifier syntax that conflicts with quotation. | Rewrite/migrate with `quoted_binding` under GRAM-14; renaming alone does not resolve the syntax conflict. |
| `complex_indexing_chain` | `let x = arr[idx1][idx2][fn()+3].member()` | Uses `fn` as an identifier, which is confusing next to the `fun` keyword. | Rewrite `fn()` to an ordinary name. |
| `doc_comment` | `/* My doc comment */ fun test() { … }` | An ordinary block comment; no doc-comment syntax exists (GRAM-16). | Rename to `block_comment_before_function`, or Rewrite once doc-comment syntax is designed. |
| `function_with_complex_return_type` | `fun getCallback(): *i32 { … }` | The return type is a simple pointer. | Rewrite with a compound type (for example a pointer to an array of owned pointers, or a function type). |
| `integer_width_edge_cases` | `let min: i8 = 0`, `let max: u64 = 0` | No edge values. | Rewrite with real bounds after AST-4 and PARSE-6 are designed. Add overflow error fixtures. |
| `lambda_expression` | `let adder = fun (b: i32) { let a = 1 + b }` | It is a lambda, but its body has no explicit result expression or return. The name `adder` suggests a semantic behavior this parser fixture does not test. | Keep as a lambda-shape test, or Rewrite with an explicit result when testing addition semantics. |
| `module` | `module test { let x = *mut i32 }` | Generic name; binds a type value using the legacy pointer spelling. | Rename (for example `module_with_type_value_binding`); Revisit under types-as-values and migrate to `&mut`. |
| `quoted_expression` | `let result = 1 + 2` | No quotation; the required backtick quotation is not implemented (GRAM-11). | Replace with real backtick quotation and `~` splice cases after boundary/nesting rules are designed; retain arithmetic only if not duplicated. |
| `quoted_expressions_with_complex_nested_structure` | `let template = { let x = 1 + 2 … return y }` | A plain block, with `return` inside an initializer. | Replace (as above). |
| `quoted_label_and_tag` | `:my_label { break :my_label }` | An ordinary label. Nothing is quoted, and there is no tag. | Rename to `labeled_block_with_labeled_break`. |
| `return_with_tag` / `return_with_tag_and_expression` | `return :label` / `return :label (3 * 4)` | These are labels, not `@` annotations/tags. | Rename to `return_with_label` / `return_with_label_and_expression`; keep annotation and quote/splice fixtures distinct. |
| `static_labeled_block_expression` | `let x = :labeled { break :labeled 5 }` | Contains no `static`. | Rename (for example `labeled_block_initializer`) to keep the labeled-break case. Add a separate Comptime fixture for a comptime-evaluated binding once `comptime.md` defines how one is expressed (for example, an initializer that calls a comptime intrinsic). |
| `static_let_with_complex_initialization` | `static let config = { … return server + ":" + port }` | Uses `return` inside an initializer and adds a string to an integer; validity depends on control-transfer and conversion rules not yet implemented. | Revisit those semantics, then Rewrite with the approved block-result and string-conversion forms; Comptime. |
| `type_value` | `let x: i64 = 10`, `let ptr: *mut i64`, `let arr: [10]i64` | Type *annotations*, not types used as values. | Rewrite to bind types as values (for example `let t = &mut i64`), or Rename to `type_annotations`; migrate pointer syntax to `&`. |
| `using_if_shortcut_with_block` | `if (…) { … } else if (interlockable()) :test { break :test 5 } else { … }` | The name is vague. Blank leading and trailing lines are hidden by trimming (TEST-3). | Rename (for example `else_if_chain_with_labeled_block`). |
| `static_block` | `static { let x = 42 let y = "hello" }` | The `name` field is snake_case and the file has a `description` field; other fixtures do not. | Comptime; normalize `name` and `description`. |
| `function_with_static` | `fun compute(x: i32): i32 { static { … } return x }` | Same naming inconsistency. | Comptime; normalize `name` and `description`. |

### Valid fixtures that match but need follow-up

| Fixture | Input (abridged) | Note | Proposed disposition |
|---|---|---|---|
| `static_binding` | `static let z = 3` | Matches. | Comptime |
| `static_function` | `static fun test(): i32 { return 42 }` | Matches. Without `static`, this function calls no intrinsic, so it would be inferred dual-mode. | Comptime: one case where the body calls an intrinsic (comptime-only), and a separate dual-mode case. |
| `static_let_with_type_and_expression` | `static let x: i32 = 1` | Matches. | Comptime |
| `quoted_binding` | ``let result = my.`-path` `` | A backtick identifier, not code quotation (GRAM-14). | Migrate/rewrite for the approved quoted-code or escaped-identifier rule; do not preserve the conflict by renaming alone. |
| `multi-line_block_comment` | `/* ... /* nesting */ ... */ let x = 1` | Already demonstrates nested-comment acceptance, but not retained text or depth. | Keep and add assertions for exact text, ranges, outer/inner nesting depth, attachment, and documentation extraction; add malformed/deeply nested cases. |
| `pointer_type` | `let ptr: *mut i64 = 20` | Assigns an integer to a pointer. The parser accepts it, but no implemented type/borrow checker decides its validity. | Revisit under the approved pointer-conversion rules and migrate to `&mut`; Merge with `mutable_pointer_modifiers` if assertions overlap. |
| `function_with_multiple_parameters_of_different_types` | `… : *own i32 { return null }` | `null` is an identifier, not a literal (GRAM-15); pointer spelling is also legacy. | Revisit after GRAM-15 and migrate the pointer type to `&own`. |
| `own_pointer_modifiers` / `owned_pointer_type`, `raw_pointer_modifiers` / `raw_pointer_type` | `let x: *own i32 = 0` vs `let x: *own i32`, … | Overlapping pointer syntax with/without an initializer. | Merge or distinguish by structural assertion, then migrate to `&own`/`&raw`. Keep mutable and immutable pointer kinds separate as `&mut` and bare `&`. |
| `string_interpolation`, `nested_string_fragments`, `string_with_multiple_interpolations` | `"Hello \(name) …"` | Overlapping interpolation cases. | Merge, or differentiate by assertion. |

### Valid fixtures that match their names (Keep)

`binary_expression`, `binding_with_type`, `binding_without_init`, `block_atom`, `block_atom_with_label`, `block_atom_with_multiple_statements`, `comments`, `complex_nested_expressions`, `complex_operation_with_multiple_operators`, `complex_pointer_types`, `deeply_nested_blocks`, `deeply_nested_if-else_expressions`, `empty_blocks_with_labels`, `function_statement`, `function_statement_with_annotation`, `grouped_expression`, `immutable_pointer_type`, `indexing`, `indexing_with_invocations_and_nesting`, `integer_type`, `invocation`, `lambda_with_multiple_parameters_and_complex_body`, `lambda_with_return_type`, `module_with_multiple_declarations`, `module_with_nested_blocks`, `multi-line_block_comment`, `multiple_annotations`, `multiple_annotations_with_different_arguments`, `multiple_annotations_with_expressions`, `multiple_break_statements_with_different_labels`, `multiple_return_statements_with_different_values`, `multiple_root_statements`, `mutable_binding`, `mutable_let_with_type_and_expression`, `mutable_pointer_modifiers`, `nested_array_types`, `nested_array_types_with_different_dimensions`, `nested_expressions`, `nested_string_fragments`, `nested_string_interpolation`, `owned_pointer_type`, `own_pointer_modifiers`, `pointer_to_array_type`, `raw_pointer_modifiers`, `raw_pointer_type`, `return_with_expression`, `simple_binding`, `string_interpolation`, `string_literal`, `string_with_multiple_interpolations`, `trailing_comma_in_annotation_arguments`, `trailing_comma_in_function_call_arguments`, `trailing_comma_in_function_parameters`, `unsized_array_type`.

The "follow-up" table above also lists some of these for merging or stronger assertions. `grouped_expression`, `nested_expressions`, and `complex_nested_expressions` currently pass even though PARSE-1 replaces their groups with `Null`. Annotation fixtures pass even though PARSE-8 drops the annotations.

### Error fixtures

| Fixture | Actual input | Problem | Proposed disposition |
|---|---|---|---|
| `incomplete_string` | `let x = \` | Corrupted: has no opening quote. Identical to the next two. | Replace with `let x = "abc`. |
| `incomplete_string_interpolation` | `let x = \` | Identical and corrupted. | Replace with `let x = "a \(b"`. |
| `unterminated_string_interpolation` | `let x = \` | Identical and corrupted. | Replace with `let x = "a \(b + c`. |
| `invalid_array_dimension` | `let x: [\` | Corrupted: has no dimension. | Replace with malformed dimension syntax such as `let x: [1 +]i32`; test negative/nonconstant sizes semantically if the approved rules reject them. |
| `mismatched_braces` | `fun test() {` | Unclosed, not mismatched. | Rename to `unclosed_brace`; add a real mismatch (`{ ]`). |
| `mismatched_brackets` | `let x = arr[1` | Unclosed, not mismatched. | Rename to `unclosed_bracket`; add a real mismatch. |
| `mismatched_parentheses` | `let x = (1 + 2` | Unclosed; overlaps with the next fixture's nested unclosed group. | Rename to `unclosed_parenthesis`, or distinguish the nesting by diagnostic assertions. |
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
