# Step 1: Design documents and guided review

> Part of the [comptime-reflection plan](./index.md). **Review gate**: this step becomes Done only after the maintainer explicitly approves every design document.

## Objective

Produce a coherent set of design documents in `docs/designs/` that settle how Patina becomes a comptime-metaprogramming language with static reflection, incremental compilation/reflection, an explicit borrow checker, and an RVSDG IR backed by `hecs` entities and components. Specify a non-ECS AST, retain nested comments for documentation, and define the required compiler syntax: backtick quotation, `~` splicing, `@symbol` annotations, and `&` pointer prefixes. Decide the Pest-aligned printing and dependency-upgrade strategy, including the direct `thiserror` to `derive_more` replacement. The designs must resolve, or explicitly defer with a reason, every finding in [`findings.md`](./findings.md) and every item in the [phase-separation carry-over](./index.md#phase-separation-carry-over). Fixed constraints are not deferral options.

Then guide the maintainer through reviewing the designs **one document at a time**, asking **one question at a time**. Record each decision in the document's `Decisions` section and in this step's review log. No compiler code, grammar, or fixture is changed in this step (constraint C-4).

## Prerequisites

- None. This is the first step.
- Inputs:
  - The [plan index](./index.md): goals, constraints C-1 to C-10, work areas, carry-over, and terminology.
  - The [findings](./findings.md): finding IDs, design-owner keys, and the fixture audit.
  - Existing documentation, starting with [`docs/developer/architecture/overview.md`](../../developer/architecture/overview.md).

## Tasks

### 1.1 Baseline from the developer documentation

Read these in order. For each, record in the review log which claims match the code and which do not, citing source paths and finding IDs. Do not promote a historical probe, old approval, or documentation assertion into a current fact. The plan's source-checked baseline is a starting point, not a substitute for checking the then-current source. Do **not** edit the inputs in this task.

- [ ] [`docs/developer/architecture/overview.md`](../../developer/architecture/overview.md) (primary baseline: pipeline, components, data flow, generics references in DOC-8)
- [ ] [`docs/developer/ast.md`](../../developer/ast.md) (DOC-7, VIS-2, AST-7, AST-10, C-8)
- [ ] [`docs/developer/architecture/visitor-pattern.md`](../../developer/architecture/visitor-pattern.md) (DOC-3, VIS-1 to VIS-3)
- [ ] [`docs/developer/testing.md`](../../developer/testing.md) (DOC-5, TEST-*)
- [ ] [`docs/developer/getting-started.md`](../../developer/getting-started.md) (DOC-6)
- [ ] [`docs/status/current.md`](../../status/current.md) (DOC-1)
- [ ] The [phase-separation carry-over](./index.md#phase-separation-carry-over) and [documentation removal record](./findings.md#removed-documentation) (DOC-2, DOC-8, DOC-11, INC-8). Do not require the deleted documents as inputs.
- [ ] The remaining RFCs in [`docs/rfcs/`](../../rfcs/): error handling, documentation, testing, and developer experience. Reconcile their examples and proposed implementation order with the active plan rather than treating them as approved designs.
- [ ] Recheck `src/lib.rs`, `src/main.rs`, `src/grammar.pest`, `src/parser.rs`, `src/ast/`, and `src/analysis/` against the implementation-boundary claims: no IR/regions, no borrow checker, no retained comments/depth, and no quote/splice nodes. Record discrepancies rather than assuming a named type or passing parse fixture implements a feature.
- [ ] Read `Cargo.toml`, `Cargo.lock`, and `.github/workflows/ci.yaml`; refresh the dated dependency inventory (DEP-1, DEP-2) and [Pest printing investigation](./findings.md#pest-printing-investigation). Distinguish semver requirements, locked versions, candidate releases, MSRV compatibility, and API changes. No dependency installation or upgrade is part of this design step.

### 1.2 Write the design documents

Create `docs/designs/` if needed, and create each document there with these sections: **Context**, **Goals and non-goals**, **Design**, **Alternatives considered**, **Findings addressed** (a table of finding ID → resolution or deferral), **Open questions**, **Decisions** (filled in during review), and **Status** (Draft / In review / Approved).

Write them in this order, because later documents depend on earlier ones. Present each document for review (task 1.3) before writing the next, unless the maintainer asks for several at once.

- [ ] **`compiler-architecture.md`** (owner key ARCH). The target pipeline from source files to RVSDG IR and the comptime evaluator. Covers: the compiler session/database; parser, owned AST, resolution, type checking, borrow checking, RVSDG lowering and evaluation boundaries; where reified code is reanalyzed before execution; `hecs` world ownership for the IR and semantic-analysis entities, one component per analysis result where appropriate, and removal of all AST ECS coupling (C-8); source ranges and file identity, preserving filenames for user-facing diagnostics (POS-5, POS-6); documentation extraction from retained AST comments/annotations; the driver (POS-3, POS-4, IR-2); and which modules survive. Explicitly distinguish the absent IR/region implementation from the target architecture (IR-1, IR-3). Include a dependency migration section (DEP-1, DEP-2): upgrade/remove decisions, `derive_more` replacing direct `thiserror`, compatible Pest versions, breaking APIs and RON fixture formats, project MSRV/CI policy, lockfile updates, and regression gates. Starts from `overview.md` and states which parts are superseded.
- [ ] **`comptime.md`** (CT). Defines the new model; the superseded stage-levels design has been deleted (DOC-2). Covers:
  - retiring the legacy `static` forms without replacement syntax (C-2, META-7): no `comptime` keyword and no `@comptime` annotation;
  - **comptime/runtime inference** (C-2, META-11):
    - the set of built-in comptime intrinsics (reflection, reification, and any others), how they are named and reached, and whether user code can extend the set;
    - the inference rule: a function that calls an intrinsic is comptime-only, comptime-only status propagates transitively to callers, and a function that reaches no intrinsic is dual-mode;
    - recursion and call-graph cycles (strongly connected components), indirect calls through function values, and calls inside reified code;
    - what triggers comptime evaluation when nothing marks it, such as type positions or a value required by a comptime-only call;
    - how the bindings and blocks formerly written `static let` and `static { }` are expressed;
    - the diagnostic when runtime code would call a comptime-only function, with the chain of calls that made it comptime-only;
  - optional dead-code elimination as a later optimization only (META-12): possible roots and reachability rules, with no correctness requirement and no requirement to ship DCE;
  - **no generics**: comptime functions that take and return types replace generic functions and parameterized `struct`, `enum`, `interface`, and `impl` (C-1, META-6, GRAM-7, ANL-6), with worked examples of a container, a constrained function, and an interface implementation;
  - types as comptime values;
  - **rethinking and renaming "stage"** (ANL-10): separating three concepts that the existing `Stage(u32)` conflates, namely a function's **mode**, an item's **sealed** (fully reified) state, and the **reification order** between items; choosing a name for each, or deciding that the order needs no name, with the rejected candidates and their collisions recorded (for example, *phase* collides with compiler passes and the removed phase-separation plan, *level* with optimization levels, and *generation* with generated code); whether any numbering exists beyond display; and how modes relate to the reification order;
  - shadowing policy across the reification order (ANL-2);
  - value availability between comptime and runtime, which comptime results can be lifted into runtime code and how, serialization, and preventing comptime pointers from escaping (ANL-3);
  - the contract for general annotations: what they mean, how they attach to targets, and whether comptime code can reflect on them; annotations carry no comptime/runtime meaning (META-8, PARSE-8, AST-1);
  - whether operators and precedence are fixed or comptime-extensible (AST-3);
  - removing the two `Stage` types and the stage-aware symbol-table, context, and type APIs in favor of the concepts above (AST-6, ANL-10);
  - the fate of phase-separation stage polymorphism.
- [ ] **`static-reflection.md`** (REFL). The reflection handles (types, declarations, fields, functions, modules) and the facts they expose. Also covers:
  - the reification API for creating items, expressions, and types;
  - the semantics of **backtick quotation and `~` splicing**, including valid splice contexts, nesting levels, typed code fragments, hygiene, and interaction with general annotations (GRAM-11, GRAM-14, AST-5, ANL-7); token choice and the presence of quotation/splicing are already fixed by C-3;
  - hygiene from the first generated binding (META-4);
  - generated-item identity and provenance (META-5, POS-5);
  - how generated items reenter analysis (META-3);
  - tracked comptime inputs, including files and environment (C-6, INC-6);
  - how derive-style features become ordinary comptime functions.
- [ ] **`incremental-compilation.md`** (INCR). Covers:
  - the query database and revision model;
  - stable `FileId`, `ModuleId`, and `ItemId` that do not depend on allocation (INC-1, INC-2, AST-7);
  - the item and module ownership index (AST-8, INC-4);
  - splitting each item's facts into signature and body;
  - fingerprints and early cutoff (INC-3);
  - dependency recording, including failed name lookups;
  - per-query diagnostics (INC-7, ANL-8);
  - the call graph and comptime/runtime mode as per-item queries, with early cutoff so that a body edit which does not change a function's mode does not invalidate its callers (META-11);
  - optional reachability/DCE queries only if a later optimization implements them (META-12); DCE is not part of correctness;
  - spans and ordinary comments kept separate from semantic fingerprints (POS-5), with documentation extraction as a separately tracked query; a comment-only edit must refresh documentation without invalidating unrelated semantic facts, unless that comment is an explicit reflected/documentation input;
  - how the shared mutable analysis context is replaced (ANL-5);
  - ownership, lifetime, and revision replacement of per-file source text and the AST (a non-ECS owned tree with retained comments), and the `hecs`-backed RVSDG IR, separating IR entity handles from stable compiler identities (C-8, C-10);
  - borrow-checker queries and summaries, invalidation after changes to a callee's borrow/escape behavior, and reuse of unrelated checked items (ANL-12);
  - whether to build or adopt a query framework (for example `salsa`), and why;
  - persistence across compiler runs, whether in scope now or deferred.
- [ ] **`incremental-reflection.md`** (IREFL). This may be merged into `static-reflection.md` or `incremental-compilation.md` if the maintainer prefers. Uses the names `comptime.md` chose for the concepts that replace "stage" (ANL-10). Covers:
  - a precise definition of a **fully reified** (sealed) item (META-9);
  - how the reification order is derived from reflection and reification dependencies, and the guarantee that comptime reflection sees only sealed items;
  - cycle detection, and diagnostics for reflecting an item that is not yet sealed;
  - reflection queries keyed by stable item identity;
  - invalidation along the reification order, so that a change to a sealed item recomputes only the later results that depended on it;
  - how a change in an item's inferred mode or position in the order propagates to its callers and to reflection results, and how this interacts with generated items;
  - stable identity for generated items across revisions, and removal of stale generated items;
  - worked examples of a single-file edit at each point in the order, including an item reified in the first round and reflected in the second.
- [ ] **`syntax-and-parsing.md`** (SYN). The required syntax corrections and otherwise minimal grammar changes (C-3) needed to resolve GRAM-1 to GRAM-18 and PARSE-1 to PARSE-10:
  - atomic tokens and keyword boundaries;
  - float literals, string escapes, and labels and tags;
  - `case`, `for`, assignment, and block separators;
  - removing generic parameter lists and dead rules;
  - preserving both `@symbol` and `@symbol(args)`, qualified names, ordering, arguments, and target attachment; fix token boundaries without treating annotations as quotes, splices, or comptime-mode markers;
  - retiring legacy `static` forms without replacement mode-marker syntax;
  - migrating pointer type prefixes to `&own`, `&raw`, `&mut`, and bare `&`, leaving `*` for multiplication and testing their distinct parses;
  - dedicated **backtick quotation and `~` splice rules** coordinated with REFL: quotation extent and delimiters, nested quotations/splices, escaping literal backticks, splice precedence, comments/strings inside quoted code, malformed/unbalanced delimiters, and migration of the conflicting `ident_backtick` rule and generic `~` operator uses (GRAM-11, GRAM-14);
  - preserving all line/block comments in the AST, not just comments that appear before a declaration; account for leading/trailing/interstitial comments, comments among annotations, file-only comments, EOF trivia, and comments nested in expressions;
  - **explicit nested block-comment depth** (GRAM-17, C-10): zero outside, increment/decrement at delimiters, close only at zero, retain AST nesting/depth, and diagnose EOF at nonzero depth and excessive nesting without panicking; compare compound-atomic Pest rules and explicit trivia collection so implicit comment skipping does not lose contents; keep strings and interpolation contexts correct;
  - documentation-comment classification, attachment to declarations/modules, treatment of unattached comments, and extraction of ordered documentation text from retained AST data (AST-10), without assuming an existing doc-comment syntax;
  - `null`;
  - error-recovery policy, so that no input panics and every construct is either faithful or rejected (C-5);
  - the source-printer contract (PRINT-1 to PRINT-3): use the versioned Pest API evidence, distinguish source replay from canonical printing, and compare retained source spans/trivia with a lightweight token/CST view; recommend lossless replay for unchanged syntax plus a grammar-aligned emitter for changed/generated AST nodes, with shared syntax/precedence rules rather than a second incompatible language; do not assume `Pairs::Display`, `concat()`, or `to_json()` is an unparser;
  - position and filename threading (POS-1, POS-2), producing a source range for every node and parse error from the Pest span's start and end positions (POS-6), and retaining source-file identity/filename resolution so user-facing diagnostics include filenames.

  For each change, list the affected fixtures from the fixture audit.
- [ ] **`ast-and-ir.md`** (ASTIR). The IR is required to be an RVSDG backed by `hecs` entities/components; semantic analyses may attach a separate result component to each relevant analysis entity; the AST **must not use ECS at all** (C-8). The current tree still has an ECS position side table, while the IR and its regions do not yet exist. Covers:
  - removing `Ast::world`, `component_entities`, the unused `Ast<C>` marker, and per-node ECS `entity` fields, with ordinary Rust source-range and metadata storage; do not retain ECS as an alternative AST design (C-8, AST-6, AST-7);
  - AST node changes: removing the legacy `Static` node and `is_static` flags without replacement mode syntax (AST-2); literal representation (AST-4); dedicated quotation/splice nodes (AST-5), and their distinction from comptime reflection values;
  - preserving general annotation names, arguments, targets, ordering, and spans through traversal and printing (AST-1, PARSE-8);
  - AST-owned comments with kind, text, full ranges, depth/nesting, and explicit source-order/attachment rules; retain unattached comments and give documentation extraction and printing access without ECS or reparsing discarded trivia (AST-10, C-10);
  - the traversal model, including enter/exit hooks, context access, and traversal of comments/annotations/quoted syntax; retire ECS component filters while preserving useful non-ECS traversal behavior and resolving VIS-1 to VIS-3;
  - the new RVSDG schema for operations and structural nodes owning **nested regions**, region membership, typed ports, arguments/results, and value/state edges; define cross-region routing, branch alternatives, loop-carried values/state, effect ordering, and malformed-graph diagnostics (IR-1, IR-3, C-8);
  - lowering resolved AST items into the RVSDG, distinguishing runtime values from type/meta values, preserving source and generated-code provenance, source ranges, and filenames for diagnostics, and representing boundaries between reification-ordered items (ANL-10) and calls used by mode inference (IR-1, META-11);
  - how comptime and runtime versions of a dual-mode function are represented in the RVSDG: one body shared by both modes, or a separate version per mode;
  - ECS world ownership and graph mutation rules, mapping stable item and graph identities to allocation-local entities, and per-item invalidation without rebuilding unrelated regions (C-8, INC-2, INC-3);
  - replacing `Arena<T>`, `Id<T>`, and the tag aliases with a coherent scheme for symbols/scopes/types and stable query IDs; define how analysis entities and per-analysis `hecs` components relate to the IR; decide whether types need an interner (ANL-1, ANL-9);
  - how type-checking results are stored and keyed; the analyses themselves are designed in `semantic-analysis.md` (ANL-4);
  - preserving resolved place/projection identity, memory effects, calls, and scope/escape information needed by the borrow checker through lowering and RVSDG region boundaries; distinguish lifetime constraints from graph regions (ANL-12);
  - how the comptime evaluator executes RVSDG operations and regions, including state dependencies and tracked effects (META-1, C-6, C-8).
- [ ] **`semantic-analysis.md`** (SEM). The required static analyses, including a borrow checker, none of which currently run on parsed programs (C-9, ANL-11, ANL-12). For each, define inputs, query/key, fixed-point strategy (call-graph strongly connected components or RVSDG region dataflow), ordering relative to comptime evaluation, reflection exposure, and source-range diagnostics. Covers:
  - name resolution and type checking, including types computed by comptime functions, with type checking interleaved with comptime evaluation (ANL-4);
  - an explicit **borrow-checker design** for `&own`, `&raw`, `&mut`, and `&`: resolved places/projections, initialization and partial moves, use-after-move, shared/exclusive loans, reborrows, writes/drops while borrowed, escaping and returned borrows, calls and closure captures, branch joins and loop fixed points, and conversions between pointer kinds;
  - raw-pointer guarantees and caller obligations, rather than blindly adopting Rust's rules; loan/lifetime facts as queries, call summaries, invalidation, and diagnostics citing both the conflicting access and the originating borrow/move;
  - applying the checker to comptime and runtime bodies and to reified code before execution, with prevention of comptime-pointer escape; graph-region nesting does not by itself prove borrow lifetimes;
  - mode inference and per-mode reachability, cross-referenced with `comptime.md` and `incremental-compilation.md` (META-11, META-12);
  - **effect tracking** through RVSDG state edges, including which effects comptime code may perform (C-6);
  - well-formedness, typing, and hygiene of generated code before it reenters the program (META-3, META-4);
  - reification-order and cycle checks (ANL-10, META-9);
  - initialization, exhaustiveness of `case`, and unreachable-code checks;
  - the order and interleaving of all analyses, and which ones can run in parallel;
  - performance budgets, and which analyses must report early cutoff so that edits stay incremental (INC-3);
  - what is deliberately out of scope, and why.
- [ ] **`testing-strategy.md`** (TST). Covers:
  - the fixture schema with expected structure and expected diagnostics (TEST-1, TEST-2, TEST-6);
  - preserving input verbatim (TEST-3);
  - **a decision on every proposed disposition in the [fixture audit](./findings.md#fixture-audit)**;
  - the naming convention, so that each fixture's name says what it tests;
  - whether to adopt snapshot testing (`insta`), or stop documenting it (TEST-9, DOC-5);
  - parse/print/parse structural equality on actual fixture inputs, plus exact byte equality for source replay; changed/generated AST cases must exercise the emitter rather than simply returning the original source (TEST-4, PRINT-2, PRINT-3);
  - CLI tests (TEST-5);
  - the coverage gaps in TEST-10;
  - **incremental-equivalence tests**, where an incremental rebuild must match a clean build for the same input;
  - query-reuse assertions, where an edit to one item must not recompute unrelated items;
  - reflection and generated-code tests;
  - general annotation parsing and retention tests at every supported location;
  - backtick quotation, nested quote/splice boundaries, `~` splicing, both annotation forms, and malformed syntax; prove annotations remain distinct from quotations;
  - comment retention/attachment and documentation extraction tests, with exact text/ranges/order, depth 0 outside and depths 1/2/3 inside nested block comments, partial closers, unterminated and excessive nesting, comment-only files, trailing/interstitial comments, strings containing comment delimiters, and print/reparse preservation;
  - comment-only edit tests: documentation is refreshed while unrelated semantic query results are reused;
  - **comptime inference tests**: direct intrinsic calls, transitive propagation, dual-mode functions, recursion and cycles, indirect calls, and a rejected runtime call to a comptime-only function;
  - if dead-code elimination is implemented as an optional optimization, behavior-equivalence tests showing enabled and disabled builds have identical observable semantics (META-12); DCE is not a correctness gate;
  - **mode-change invalidation tests**: adding or removing an intrinsic call recomputes exactly the affected callers;
  - **RVSDG tests**: build previously nonexistent regions for functions, alternatives, and loops; validate membership, argument/result arity/types, cross-region edges, and value/state routing; test lowering/evaluation, effect ordering, and unchanged-item reuse (C-8, IR-3);
  - **borrow-checker tests**: accepted/rejected moves, partial moves, shared/mutable loans, reborrows, lifetime escapes, drops, branches/loops, calls/captures, and pointer-kind conversions, including explicit raw-pointer boundaries; run the cases at comptime/runtime and after reification, assert diagnostic ranges, and test callee-summary invalidation plus clean/incremental equivalence (ANL-12);
  - **semantic-analysis tests** for the remaining analyses in `semantic-analysis.md`, with expected diagnostics and source ranges (C-9, POS-6);
  - dependency upgrade gates on the selected MSRV/stable toolchains: all fixtures deserialize, parser/printer behavior stays intentional, Pest source formatting is version-appropriate, and locked builds/tests/clippy remain reproducible (DEP-1, DEP-2);
  - **source-range tests**: the start and end line/column of nodes and diagnostics, including multi-line constructs (POS-6).

### 1.3 Guided review

Before presenting any document, complete [agent review](../README.md#agent-review) of all the design documents with a different-model subagent, and resolve every High and Medium comment.

- [ ] Agent review is complete for all design documents: no High or Medium comment is unresolved, and each round is recorded in this step's review log.

For each design document, in the order above:

- [ ] Present a short summary and the list of open questions.
- [ ] Ask the maintainer one question at a time using the `ask_user` tool, preferring multiple-choice options with a recommended choice.
- [ ] Record each answer in the document's `Decisions` section and in this step's review log, then revise the document to match.
- [ ] Set the document's status to **Approved** only after the maintainer explicitly approves it. Revisit earlier documents if a later decision invalidates them.

### 1.4 Reconcile existing documentation

Change only the documentation status needed to keep it truthful. Substantive rewrites of user and developer docs are planned in Step 2 under WA-9.

- [ ] Update navigation and references to point to the new designs. Keep the removed documents out of prerequisites, and retain their relevant findings and carry-over dispositions.
- [ ] Reconcile references to the removed AST-storage finding with C-8; do not reintroduce it or describe ECS-backed AST storage as an alternative. Keep source/doc corrections outside this plan-only revision assigned to WA-9.
- [ ] Add the new design documents to [`docs/README.md`](../../README.md).
- [ ] Confirm that every finding ID and every carry-over row appears in at least one design's **Findings addressed** table, either resolved or explicitly deferred. Record the check in the review log.

## Acceptance criteria

- Every design document listed in task 1.2 exists in `docs/designs/` (or was merged into another with the maintainer's approval) and has status **Approved**.
- Every finding ID in [`findings.md`](./findings.md), and every row of the phase-separation carry-over table, is resolved or explicitly deferred with a reason in some design's **Findings addressed** table.
- Every proposed fixture disposition has a decision recorded in `testing-strategy.md`.
- The designs satisfy constraints C-1 to C-10: no generics or comptime-mode markers; required quote/splice/annotation and `&` pointer syntax; designs before code; faithful parsing; tracked effects; preserved history; no ECS in the AST and `hecs` use for the IR/semantic analysis; incremental analysis including a borrow checker; and retained comments with explicit nesting depth.
- The legacy "stage" concept is split into mode, sealing, and reification order, each with a chosen name (or an explicit decision that it needs none), and the rejected names are recorded (ANL-10).
- Every location in the designs is a source range from a start line/column to an end line/column (POS-6).
- `semantic-analysis.md` defines each analysis as an incremental query, including the borrow checker's loan/move/lifetime model, raw-pointer boundary, fixed points, call summaries, diagnostics, and application to comptime/runtime/reified code (ANL-11, ANL-12).
- Comments, nested depth, and annotation data remain in the non-ECS AST; worked examples show documentation extraction and printing without losing text or attachment.
- The grammar design fixes backticks as quoting, `~` as splicing, both `@symbol` annotation forms, and `&own`, `&raw`, `&mut`, and bare `&` as pointer prefixes, with explicit migration of legacy backtick identifiers, `*` pointer forms, and generic `~` operator handling.
- The printer decision cites verified Pest versions/APIs and distinguishes exact source replay from generation; acceptance tests cover actual parse/print/parse structure, retained trivia, and changed/generated ASTs.
- The dependency migration matrix distinguishes manifest, locked, and candidate versions, specifies MSRV/API/fixture-format changes, and schedules upgrades or removals under WA-11 without making them in Step 1.
- The designs define the built-in comptime intrinsic set, the inference rule and its handling of cycles and indirect calls, and evaluation triggers and value lifting. DCE is explicitly optional and not required for correctness.
- The designs specify the RVSDG entity/component model, per-analysis components on relevant analysis entities, region and port invariants, value/state dependencies and effect ordering, lowering and evaluation, and stable incremental identities distinct from ECS entity handles. They include worked examples of calls, branching, loops, and an effectful operation.
- The diagnostic/source design preserves source-file identity and resolves filenames for user-facing diagnostics; source-range changes must not erase filenames.
- The designs agree with each other. Terminology matches the [plan index](./index.md#terminology), and later decisions have been propagated to earlier documents.
- No files under `src/` or `test_cases/`, or `Cargo.toml`/`Cargo.lock`, were changed by this step.
- The review log records the baseline observations, every decision, and the maintainer's approval of each document.

## Status

Not started

## Review log

- **2026-11-04 — Plan revision only, not execution or approval of Step 1.** Rechecked current-state claims against source and fixtures. C-8 requires a non-ECS AST and ECS-backed RVSDG IR plus semantic-analysis components, with regions explicitly unimplemented. Added retained comments/depth, documentation extraction, compiler-defined backtick/`~`/`@` syntax, `&` pointer prefixes, versioned Pest printing research, dependency modernization including direct `thiserror` → `derive_more`, and an explicit incremental borrow checker. DCE is optional and not a correctness requirement. Preserved filename resolution for diagnostics and corrected overbroad span, traversal, fixture, and documentation claims. Status remains **Not started**; no design documents were created or approved.
