# Step 2: Implementation planning

> Part of the [comptime-reflection plan](./index.md). **Review gate**: this step becomes Done only after the maintainer explicitly approves the generated implementation steps.

## Objective

Convert the approved design documents from [Step 1](./step-1.md) into concrete, ordered implementation steps (`step-3.md` through `step-N.md`) in this directory. Each step must be small enough to complete and verify in a focused session. Each implementation step leaves `cargo fmt --all --check`, `cargo build --locked`, `cargo test --locked`, and `cargo clippy --locked --all-targets --all-features -- -D warnings` passing, and advances a **vertical slice** of the [dependency chain](./index.md#dependency-chain). Do not complete one layer in isolation before starting the next.

This step writes plan files only. It does not change compiler code, grammar, fixtures, `Cargo.toml`, or `Cargo.lock`.

## Prerequisites

- [Step 1](./step-1.md) is **Done**: every design document is Approved.
- Inputs: the approved designs in `docs/designs/`, the [findings](./findings.md), and the [work areas](./index.md#work-areas).

## Tasks

- [ ] Re-read the approved designs and their **Decisions** sections. If any decision conflicts with the plan index (goals, constraints, work areas, terminology), stop and resolve it with the maintainer before planning.
- [ ] Revalidate implementation claims against then-current source and locked dependencies. Do not assume regions, comment retention/depth, a printer unparser, or a borrow checker exists because it appears in a design.
- [ ] Draft the ordered list of implementation steps. Follow the dependency chain, and interleave fixture truthfulness (WA-1) and documentation accuracy (WA-9) with each slice. Expected early steps include:
  - [ ] **Test harness first.** Implement the fixture schema and assertions decided in `testing-strategy.md`, and apply the fixture-audit dispositions that do not depend on grammar changes. Later grammar changes then have truthful regression tests.
  - [ ] **Dependency modernization** (WA-11, DEP-1, DEP-2). Refresh available versions and implement the approved manifest/lockfile upgrades or removals. Replace direct `thiserror` with `derive_more` for error derives; a transitive `thiserror` may remain only if another dependency requires it. Separate compatible lockfile refreshes from breaking `hecs`/RON/error-library migrations; keep Pest and its derive/generator/meta stack compatible. Validate MSRV, RON deserialization, parser/printer behavior, and CI flags. Do this before relying on new Pest APIs, and keep dependency changes separate from intentional language changes so regressions are attributable.
  - [ ] **Legacy `static` retirement and generics removal** (WA-4), applied consistently across the grammar, AST, analysis, fixtures, and docs. Remove the `static` forms without adding a `comptime` keyword or a `@comptime` annotation. Rewrite or replace the `static_*` fixtures as `testing-strategy.md` decides. Grammar work covers only the retirement of `static` and generics removal; general annotation parsing is unchanged.
  - [ ] **Grammar and parser fidelity** (WA-2) together with **AST, spans, printer, and visitors** (WA-3). Remove user-input panics/placeholders, including conditionless `if` conversion (PARSE-10). Remove all AST ECS storage/handles under C-8, retaining the owned tree and ordinary source-range/metadata fields. Extend current byte spans with the approved full source-range model in parser, AST, diagnostics, and symbols (POS-6), retaining source-file identity and filenames in rendered diagnostics. Migrate pointer type prefixes to `&own`, `&raw`, `&mut`, and `&`; keep `*` as multiplication. Migrate CLI counts, tests, and traversal rather than leaving ECS-only consumers behind.
  - [ ] **Comments and documentation slice** (WA-2, WA-3, GRAM-16, GRAM-17, AST-10, C-10). Retain line/block comments at all trivia positions in AST-owned data. Track nested block depth and preserve nesting/ranges/text/order; diagnose unterminated and excessive nesting. Implement the approved documentation classification/attachment and extraction path, with end-to-end source-to-documentation tests, comment-only input, depth assertions, and printer preservation.
  - [ ] **Quotation/splicing and annotations** (GRAM-11, GRAM-14, AST-1, AST-5, PARSE-8). Implement dedicated backtick quote and `~` splice grammar/AST nodes; migrate conflicting backtick identifiers and generic `~` handling. Preserve `@symbol` / `@symbol(args)` including qualified names and expression arguments. Test nesting, binding, escaping, malformed delimiters, and retention through traversal/printing, without adding comptime-mode syntax. Syntax can land before its evaluator only with an explicit not-yet-supported semantic diagnostic.
  - [ ] **Pest-aligned printing** (PRINT-1 to PRINT-3, TEST-4). Implement the approved source-replay and changed/generated-AST emitter paths on the selected Pest version. Preserve comments/annotations and correct quoting, splicing, escaping, labels, and precedence. Replace name-selected mock fixture assertions with real parse/print/parse structural tests, byte-exact replay tests, and generated/modified-node cases.
  - [ ] **Incremental foundations** (WA-6) before semantic queries. Introduce stable IDs and the query database, and re-express parsing and item collection as queries.
  - [ ] **Stage rename** (WA-8, ANL-10) before or with the first semantic queries. Remove `analysis::Stage`, the ECS `Stage` component, and the stage-aware symbol-table, context, and type APIs, replacing them with the mode, sealing, and reification-order concepts and names chosen in `comptime.md`. Update the docs and `AGENTS.md` in the same step.
  - [ ] **Semantic pipeline and RVSDG IR** (WA-5) built as queries. Replace incompatible ID/storage APIs with the approved scheme (ANL-1, ANL-9). Keep the AST outside ECS; use `hecs` for the IR and semantic analysis, with a distinct component for each analysis result attached to the relevant entity where appropriate. Add resolution, type checking, call-graph and mode queries. Construct new `hecs`-backed RVSDG operations and **regions** for function bodies, alternatives, and loops, with membership, arguments/results, typed ports, explicit boundary routing, value/state edges, and validators (IR-1, IR-3, C-8). Preserve place/projection and memory-effect information for borrow checking; do not treat AST blocks or symbol scopes as graph regions.
  - [ ] **Borrow checker** (WA-10, ANL-12, C-9) as an explicit implementation slice or series of named slices, not an implicit subtask of type checking. Build move/initialization/loan/lifetime queries and call summaries over the approved resolved/RVSDG representation. Cover shared/exclusive borrows, reborrows, alias/write/drop conflicts, partial moves, escaping lifetimes, captures/calls, branches/loops, and all pointer conversions including the raw-pointer safety boundary. Require positive/negative programs with multi-site source-range diagnostics and incremental reuse/invalidation tests.
  - [ ] **Other semantic analyses** (WA-10, C-9, ANL-11) in the design's order, with effects and generated-code checking interleaved with the borrow checker. Each analysis is an incremental query with positive/negative programs and diagnostics, not a whole-program rerun.
  - [ ] **RVSDG comptime evaluator, reflection, and reification** (WA-7), followed by **incremental reflection** (WA-8). Depend on the applicable region validators, borrow checker, and effect checks before executing code. Reified code reenters resolution, typing, borrowing, and effect analysis; test that generated code and comptime pointers cannot bypass safety checks. Extend checker coverage alongside new executable constructs.
  - [ ] **Optional dead-code elimination** (META-12), only if selected as a later optimization. It is not required for correctness or an acceptance gate; test that enabling it preserves observable behavior.
- [ ] For each step, create `step-<n>.md` using the [step file format](../README.md#step-file-format). Each step file must:
  - cite the finding IDs it resolves and the design sections it implements;
  - list concrete tasks naming the files and modules to change;
  - include acceptance criteria with specific tests. Where relevant, these are end-to-end tests from source to result, and incremental-equivalence or query-reuse tests as defined in `testing-strategy.md`;
  - state whether it is a review gate. Steps that change the language surface or a public compiler API should be review gates.
- [ ] Update the [status table](./index.md#status) in `index.md` with every new step, its review-gate flag, and its prerequisites. Remove the "3+ *Created by Step 2*" placeholder row.
- [ ] Verify coverage: every finding ID and every carry-over row is assigned to exactly one implementation step, or is listed as explicitly deferred (with the design that deferred it). Record the mapping in the review log, or as a table in `index.md`.
- [ ] Complete [agent review](../README.md#agent-review) of the generated step files and index changes with a different-model subagent. Resolve every High and Medium comment, and record each round in this step's review log.
- [ ] Present the step list to the maintainer for review, one step at a time, asking one question at a time. Record the decisions, revise, and then obtain explicit approval.

## Acceptance criteria

- `step-3.md` through `step-N.md` exist and follow the step file format.
- Every step cites findings and design sections, and has verifiable acceptance criteria that include tests.
- The order respects the dependency chain. No step depends on a later step.
- The IR steps implement the approved RVSDG design using `hecs` entities and components, with tests for graph invariants, lowering/evaluation semantics, effect ordering, and incremental reuse.
- The IR steps explicitly create the currently missing RVSDG region model, including membership, boundary ports, arguments/results, alternatives/loops, and malformed-graph diagnostics (IR-3); the AST never uses ECS in the target design, while semantic-analysis results may use separate `hecs` components.
- Comment retention and depth, AST-based documentation extraction, backtick quotation, `~` splicing, both annotation forms, and grammar-aligned printing have concrete tasks and acceptance tests.
- Dependency steps name manifest and lockfile changes, version/MSRV/API migrations, and behavior-focused regression checks; newer Pest formatting is never assumed on an older locked release.
- Dependency steps replace direct `thiserror` use with `derive_more`, updating derives and tests while allowing transitive `thiserror` only when required by another dependency.
- The semantic-analysis steps cover every analysis in `semantic-analysis.md`, each as an incremental query with accepted and rejected programs (C-9), and explicitly name the borrow checker, its `&own`/`&raw`/`&mut`/`&` pointer-kind rules, diagnostics, call-summary invalidation, and comptime/runtime/reified-code coverage.
- Every finding and every carry-over row is mapped to a step or explicitly deferred.
- The `index.md` status table lists all steps accurately.
- The maintainer has explicitly approved the step list.
- No files under `src/` or `test_cases/`, or `Cargo.toml`/`Cargo.lock`, were changed by this step.

## Status

Not started

## Review log

- **2026-11-04 — Plan revision only, not execution or approval of Step 2.** Added explicit implementation-planning coverage for dependency modernization including direct `thiserror` → `derive_more`, a non-ECS AST with retained comments/depth and documentation extraction, compiler-defined backtick/`~`/`@` syntax, `&` pointer prefixes, Pest-aligned printing, new RVSDG regions, and the incremental borrow checker. DCE remains optional and non-correctness-critical; diagnostics retain filenames. Safety checks precede comptime execution and are reapplied to reified code. Status remains **Not started**; no implementation steps were generated.
