# Step 2: Implementation planning

> Part of the [comptime-reflection plan](./index.md). **Review gate**: this step becomes Done only after the maintainer explicitly approves the generated implementation steps.

## Objective

Convert the approved design documents from [Step 1](./step-1.md) into concrete, ordered implementation steps (`step-3.md` through `step-N.md`) in this directory. Each step must be small enough to complete and verify in a focused session. Each step leaves `cargo fmt --check`, `cargo build`, `cargo test`, and `cargo clippy` passing, and advances a **vertical slice** of the [dependency chain](./index.md#dependency-chain). Do not complete one layer in isolation before starting the next.

This step writes plan files only. It does not change compiler code, the grammar, or fixtures.

## Prerequisites

- [Step 1](./step-1.md) is **Done**: every design document is Approved.
- Inputs: the approved designs in `docs/designs/`, the [findings](./findings.md), and the [work areas](./index.md#work-areas).

## Tasks

- [ ] Re-read the approved designs and their **Decisions** sections. If any decision conflicts with the plan index (goals, constraints, work areas, terminology), stop and resolve it with the maintainer before planning.
- [ ] Draft the ordered list of implementation steps. Follow the dependency chain, and interleave fixture truthfulness (WA-1) and documentation accuracy (WA-9) with each slice. Expected early steps include:
  - [ ] **Test harness first.** Implement the fixture schema and assertions decided in `testing-strategy.md`, and apply the fixture-audit dispositions that do not depend on grammar changes. Later grammar changes then have truthful regression tests.
  - [ ] **Legacy `static` retirement and generics removal** (WA-4), applied consistently across the grammar, AST, analysis, fixtures, and docs. Remove the `static` forms without adding a `comptime` keyword or a `@comptime` annotation. Rewrite or replace the `static_*` fixtures as `testing-strategy.md` decides. Grammar work covers only the retirement of `static` and generics removal; general annotation parsing is unchanged.
  - [ ] **Grammar and parser fidelity** (WA-2) together with **AST, spans, printer, and visitors** (WA-3). Remove every panic and placeholder on user input. Replace the AST's vestigial `hecs` side table with inline source ranges as `ast-and-ir.md` decides, so that the RVSDG IR is the only `hecs` user (AST-9). Replace the start-only `Position` with the approved source-range type in the parser, AST, diagnostics, and symbol table, with range tests (POS-6).
  - [ ] **Incremental foundations** (WA-6) before semantic queries. Introduce stable IDs and the query database, and re-express parsing and item collection as queries.
  - [ ] **Stage rename** (WA-8, ANL-10) before or with the first semantic queries. Remove `analysis::Stage`, the ECS `Stage` component, and the stage-aware symbol-table, context, and type APIs, replacing them with the mode, sealing, and reification-order concepts and names chosen in `comptime.md`. Update the docs and `AGENTS.md` in the same step.
  - [ ] **Semantic pipeline and RVSDG IR** (WA-5) built as queries. Start by removing `Arena<T>`, `Id<T>`, and the tag aliases in favor of the approved identity scheme (ANL-1, ANL-9). Include the call graph, per-item lowering into `hecs` entities/components, and graph validation (C-8). Then **comptime inference** as a per-item mode query over the call graph, with its diagnostics (WA-7, META-11). Then the **RVSDG comptime evaluator, reflection, and reification** (WA-7), and then **incremental reflection** (WA-8). Preserve stable item/graph identities independently of ECS allocation and cover value/state dependencies and region semantics.
  - [ ] **Semantic analyses** (WA-10, C-9, ANL-11) as a series of slices, in the order `semantic-analysis.md` decides. For example: resolution and type checking first, then ownership and borrowing for each pointer kind, then effects, then checks on generated code. Each analysis is an incremental query with accepted and rejected test programs and range-carrying diagnostics. No analysis is implemented as a whole-program pass.
  - [ ] **Dead code elimination** for comptime and runtime outputs as reachability queries (WA-7, META-12). Plan it so that each mode's output can be checked for the absence of unused versions as soon as that mode produces output.
- [ ] For each step, create `step-<n>.md` using the [step file format](../README.md#step-file-format). Each step file must:
  - cite the finding IDs it resolves and the design sections it implements;
  - list concrete tasks naming the files and modules to change;
  - include acceptance criteria with specific tests. Where relevant, these are end-to-end tests from source to result, and incremental-equivalence or query-reuse tests as defined in `testing-strategy.md`;
  - state whether it is a review gate. Steps that change the language surface or a public compiler API should be review gates.
- [ ] Update the [status table](./index.md#status) in `index.md` with every new step, its review-gate flag, and its prerequisites. Remove the "3+ *Created by Step 2*" placeholder row.
- [ ] Verify coverage: every finding ID and every carry-over row is assigned to exactly one implementation step, or is listed as explicitly deferred (with the design that deferred it). Record the mapping in the review log, or as a table in `index.md`.
- [ ] Present the step list to the maintainer for review, one step at a time, asking one question at a time. Record the decisions, revise, and then obtain explicit approval.

## Acceptance criteria

- `step-3.md` through `step-N.md` exist and follow the step file format.
- Every step cites findings and design sections, and has verifiable acceptance criteria that include tests.
- The order respects the dependency chain. No step depends on a later step.
- The IR steps implement the approved RVSDG design using `hecs` entities and components, with tests for graph invariants, lowering/evaluation semantics, effect ordering, and incremental reuse.
- The semantic-analysis steps cover every analysis in `semantic-analysis.md`, each as an incremental query with accepted and rejected programs (C-9).
- Every finding and every carry-over row is mapped to a step or explicitly deferred.
- The `index.md` status table lists all steps accurately.
- The maintainer has explicitly approved the step list.
- No files under `src/`, `test_cases/`, or `Cargo.toml` were changed by this step.

## Status

Not started

## Review log

*No entries yet.*
