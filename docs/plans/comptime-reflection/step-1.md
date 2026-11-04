# Step 1: Design documents and guided review

> Part of the [comptime-reflection plan](./index.md). **Review gate**: this step becomes Done only after the maintainer explicitly approves every design document.

## Objective

Produce a coherent set of design documents in `docs/designs/` that settle how Patina becomes a comptime-metaprogramming language with static reflection, incremental compilation, incremental reflection, and an RVSDG IR backed by `hecs` entities and components. The designs must resolve, or explicitly defer with a reason, every finding in [`findings.md`](./findings.md) and every item in the [phase-separation carry-over](./index.md#phase-separation-carry-over).

Then guide the maintainer through reviewing the designs **one document at a time**, asking **one question at a time**. Record each decision in the document's `Decisions` section and in this step's review log. No compiler code, grammar, or fixture is changed in this step (constraint C-4).

## Prerequisites

- None. This is the first step.
- Inputs:
  - The [plan index](./index.md): goals, constraints C-1 to C-9, work areas, carry-over, and terminology.
  - The [findings](./findings.md): finding IDs, design-owner keys, and the fixture audit.
  - Existing documentation, starting with [`docs/developer/architecture/overview.md`](../../developer/architecture/overview.md).

## Tasks

### 1.1 Baseline from the developer documentation

Read these in order. For each, record in the review log which claims match the code and which do not, citing finding IDs. Do **not** edit them in this task.

- [ ] [`docs/developer/architecture/overview.md`](../../developer/architecture/overview.md) (primary baseline: pipeline, components, data flow, generics references in DOC-8)
- [ ] [`docs/developer/ast.md`](../../developer/ast.md) (DOC-7, VIS-2, AST-7, AST-9)
- [ ] [`docs/developer/architecture/visitor-pattern.md`](../../developer/architecture/visitor-pattern.md) (DOC-3, VIS-1 to VIS-3)
- [ ] [`docs/developer/testing.md`](../../developer/testing.md) (DOC-5, TEST-*)
- [ ] [`docs/developer/getting-started.md`](../../developer/getting-started.md) (DOC-6)
- [ ] [`docs/status/current.md`](../../status/current.md) (DOC-1)
- [ ] The [phase-separation carry-over](./index.md#phase-separation-carry-over) and [documentation removal record](./findings.md#removed-documentation) (DOC-2, DOC-8, DOC-11, INC-8). Do not require the deleted documents as inputs.
- [ ] The remaining RFCs in [`docs/rfcs/`](../../rfcs/): error handling, documentation, testing, and developer experience. Reconcile their examples and proposed implementation order with the active plan rather than treating them as approved designs.

### 1.2 Write the design documents

Create `docs/designs/` if needed, and create each document there with these sections: **Context**, **Goals and non-goals**, **Design**, **Alternatives considered**, **Findings addressed** (a table of finding ID → resolution or deferral), **Open questions**, **Decisions** (filled in during review), and **Status** (Draft / In review / Approved).

Write them in this order, because later documents depend on earlier ones. Present each document for review (task 1.3) before writing the next, unless the maintainer asks for several at once.

- [ ] **`compiler-architecture.md`** (owner key ARCH). The target pipeline from source files to RVSDG IR and the comptime evaluator. Covers: the compiler session and database; the boundaries between the parser, AST, resolution, type checking, the semantic analyses of `semantic-analysis.md`, RVSDG lowering, and evaluator; where reification feeds back into analysis; world ownership and lifecycle boundaries for the `hecs`-backed RVSDG IR (C-8), and the AST's representation as a plain owned tree that is not stored in `hecs` (AST-9); the source-range type that every phase uses for locations and diagnostics, and the file table it refers to (POS-6, POS-5); the driver and CLI (POS-3, POS-4, IR-2); and which existing modules survive. Starts from `overview.md` and states which parts of it are superseded.
- [ ] **`comptime.md`** (CT). Defines the new model; the superseded stage-levels design has been deleted (DOC-2). Covers:
  - retiring the legacy `static` forms without replacement syntax (C-2, META-7): no `comptime` keyword and no `@comptime` annotation;
  - **comptime/runtime inference** (C-2, META-11):
    - the set of built-in comptime intrinsics (reflection, reification, and any others), how they are named and reached, and whether user code can extend the set;
    - the inference rule: a function that calls an intrinsic is comptime-only, comptime-only status propagates transitively to callers, and a function that reaches no intrinsic is dual-mode;
    - recursion and call-graph cycles (strongly connected components), indirect calls through function values, and calls inside reified code;
    - what triggers comptime evaluation when nothing marks it, such as type positions or a value required by a comptime-only call;
    - how the bindings and blocks formerly written `static let` and `static { }` are expressed;
    - the diagnostic when runtime code would call a comptime-only function, with the chain of calls that made it comptime-only;
  - **dead code elimination** for both modes (META-12): the roots of each mode, reachability of dual-mode functions in each mode, whether reflection or reification makes an item reachable, and the guarantee that unused comptime or runtime versions do not affect the executable binary;
  - **no generics**: comptime functions that take and return types replace generic functions and parameterized `struct`, `enum`, `interface`, and `impl` (C-1, META-6, GRAM-7, ANL-6), with worked examples of a container, a constrained function, and an interface implementation;
  - types as comptime values;
  - **rethinking and renaming "stage"** (ANL-10): separating three concepts that the existing `Stage(u32)` conflates, namely a function's **mode**, an item's **sealed** (fully reified) state, and the **reification order** between items; choosing a name for each, or deciding that the order needs no name, with the rejected candidates and their collisions recorded (for example, *phase* collides with compiler passes and the removed phase-separation plan, *level* with optimization levels, and *generation* with generated code); whether any numbering exists beyond display; and how modes relate to the reification order;
  - shadowing policy across the reification order (ANL-2);
  - value availability between comptime and runtime, which comptime results can be lifted into runtime code and how, serialization, and preventing comptime pointers from escaping (ANL-3);
  - the contract for general annotations: what they mean, how they attach to targets, and whether comptime code can reflect on them; annotations carry no comptime/runtime meaning (META-8, PARSE-8, AST-1);
  - whether operators and precedence are fixed or comptime-extensible (AST-3);
  - the open question of user-defined surface syntax (META-10);
  - removing the two `Stage` types and the stage-aware symbol-table, context, and type APIs in favor of the concepts above (AST-6, ANL-10);
  - the fate of phase-separation stage polymorphism.
- [ ] **`static-reflection.md`** (REFL). The reflection handles (types, declarations, fields, functions, modules) and the facts they expose. Also covers:
  - the reification API for creating items, expressions, and types;
  - quote and splice syntax, or the decision not to have them (GRAM-11, AST-5, ANL-7);
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
  - reachability for dead code elimination as an incremental query per mode (META-12);
  - spans kept separate from semantic fingerprints (POS-5);
  - how the shared mutable analysis context is replaced (ANL-5);
  - ownership, lifetime, and revision replacement of the per-file AST (a plain tree, AST-9) and the `hecs`-backed RVSDG IR, separating the IR's allocation-local entity handles from stable compiler identities and semantic fingerprints (C-8);
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
- [ ] **`syntax-and-parsing.md`** (SYN). The minimal grammar changes (C-3) needed to resolve GRAM-1 to GRAM-16 and PARSE-1 to PARSE-9:
  - atomic tokens and keyword boundaries;
  - float literals, string escapes, and labels and tags;
  - `case`, `for`, assignment, and block separators;
  - removing generic parameter lists and dead rules;
  - retaining the existing general `@name(args)` annotation grammar, with no comptime-specific placement or parsing rule;
  - retiring legacy `static` forms without replacement syntax, and quote or splice syntax if REFL requires it;
  - doc comments;
  - `null`;
  - error-recovery policy, so that no input panics and every construct is either faithful or rejected (C-5);
  - the source-printer contract (PRINT-1);
  - position and filename threading (POS-1, POS-2), and producing a source range for every node and parse error from the Pest span's start and end positions (POS-6).

  For each change, list the affected fixtures from the fixture audit.
- [ ] **`ast-and-ir.md`** (ASTIR). The IR is required to be an RVSDG backed by `hecs` entities and components (C-8), not an open choice of IR family or storage model. The AST is not `hecs`-backed: it is a plain owned Rust tree with a vestigial per-node `Position` side table (AST-9). Covers:
  - the AST's storage: removing the `hecs::World`, `component_entities`, the `Ast<C>` marker parameter, and per-node `entity` fields in favor of inline spans, or justifying any part that is kept, so that only the IR uses `hecs` (AST-9, AST-6, AST-7);
  - AST node changes: removing the legacy `Static` node and `is_static` flags with no replacement node (AST-2), with mode held as a derived per-item fact; literal representation (AST-4); and reflection or quote representations if required;
  - preserving general annotation names, arguments, targets, ordering, and spans through traversal and printing (AST-1, PARSE-8);
  - the traversal model, including enter/exit hooks, context access, and the `Traversable` and `ComponentFilterWrapper` defects; the component filters disappear if the AST side table is removed (VIS-1 to VIS-3, AST-9);
  - the RVSDG entity/component schema for operations, nested regions, typed ports, region arguments/results, and value/state dependencies; structural control flow, region scoping, effect ordering, and graph validation (IR-1, C-8);
  - lowering resolved AST items into the RVSDG, distinguishing runtime values from type/meta values, preserving source and generated-code provenance and source ranges, and representing boundaries between reification-ordered items (ANL-10) and calls used by mode inference and dead code elimination (IR-1, META-11, META-12);
  - how comptime and runtime versions of a dual-mode function are represented in the RVSDG: one body shared by both modes, or a separate version per mode;
  - ECS world ownership and graph mutation rules, mapping stable item and graph identities to allocation-local entities, and per-item invalidation without rebuilding unrelated regions (C-8, INC-2, INC-3);
  - replacing `Arena<T>`, `Id<T>`, and the `SymbolId`/`TypeId`/`ScopeId` tag aliases with one identity scheme for symbols, scopes, and types, built from stable query identities and `hecs` storage; whether types need an interner, given that types are comptime values (ANL-1, ANL-9);
  - how type-checking results are stored and keyed; the analyses themselves are designed in `semantic-analysis.md` (ANL-4);
  - how the comptime evaluator executes RVSDG operations and regions, including state dependencies and tracked effects (META-1, C-6, C-8).
- [ ] **`semantic-analysis.md`** (SEM). The full set of static analyses that Patina needs, which goes beyond what Rust's compiler performs (C-9, ANL-11). For each analysis, define: its inputs (resolved items or the RVSDG), the query that computes it and its key, its fixed-point strategy (over call-graph strongly connected components or region dataflow), when it runs relative to comptime evaluation, which of its results reflection may expose, and its diagnostics with source ranges (POS-6). Covers:
  - name resolution and type checking, including types computed by comptime functions, with type checking interleaved with comptime evaluation (ANL-4);
  - **ownership, aliasing, mutability, and lifetimes** for the four pointer kinds `*own`, `*raw`, `*mut`, and `*`: moves, borrows, conversions between kinds, what `*raw` gives up, and how these rules apply to comptime values and code generated by reification;
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
  - printer round-trip tests (TEST-4, PRINT-2);
  - CLI tests (TEST-5);
  - the coverage gaps in TEST-10;
  - **incremental-equivalence tests**, where an incremental rebuild must match a clean build for the same input;
  - query-reuse assertions, where an edit to one item must not recompute unrelated items;
  - reflection and generated-code tests;
  - general annotation parsing and retention tests at every supported location;
  - **comptime inference tests**: direct intrinsic calls, transitive propagation, dual-mode functions, recursion and cycles, indirect calls, and a rejected runtime call to a comptime-only function;
  - **dead code elimination tests**: unused comptime and runtime versions are absent from each mode's output;
  - **mode-change invalidation tests**: adding or removing an intrinsic call recomputes exactly the affected callers;
  - **RVSDG tests**: ECS graph construction/traversal, port and region invariants, rejection of malformed graphs, lowering/evaluation semantics, preservation of required effect ordering, and reuse of unchanged item graphs across revisions (C-8);
  - **semantic-analysis tests**: accepted and rejected programs for each analysis in `semantic-analysis.md`, especially ownership and borrowing for each pointer kind, with expected diagnostics and their source ranges (C-9, POS-6);
  - **source-range tests**: the start and end line/column of nodes and diagnostics, including multi-line constructs (POS-6).

### 1.3 Guided review

For each design document, in the order above:

- [ ] Present a short summary and the list of open questions.
- [ ] Ask the maintainer one question at a time using the `ask_user` tool, preferring multiple-choice options with a recommended choice.
- [ ] Record each answer in the document's `Decisions` section and in this step's review log, then revise the document to match.
- [ ] Set the document's status to **Approved** only after the maintainer explicitly approves it. Revisit earlier documents if a later decision invalidates them.

### 1.4 Reconcile existing documentation

Change only the documentation status needed to keep it truthful. Substantive rewrites of user and developer docs are planned in Step 2 under WA-9.

- [ ] Update navigation and references to point to the new designs. Keep the removed documents out of prerequisites, and retain their relevant findings and carry-over dispositions.
- [ ] Add the new design documents to [`docs/README.md`](../../README.md).
- [ ] Confirm that every finding ID and every carry-over row appears in at least one design's **Findings addressed** table, either resolved or explicitly deferred. Record the check in the review log.

## Acceptance criteria

- Every design document listed in task 1.2 exists in `docs/designs/` (or was merged into another with the maintainer's approval) and has status **Approved**.
- Every finding ID in [`findings.md`](./findings.md), and every row of the phase-separation carry-over table, is resolved or explicitly deferred with a reason in some design's **Findings addressed** table.
- Every proposed fixture disposition has a decision recorded in `testing-strategy.md`.
- The designs satisfy constraints C-1 to C-9: no generics, inferred comptime/runtime modes with no marker syntax, minimal grammar change, designs before code, faithful-or-rejected behavior, tracked comptime effects, preserved history, an RVSDG IR backed by `hecs` entities/components, and deep, incremental semantic analysis.
- The legacy "stage" concept is split into mode, sealing, and reification order, each with a chosen name (or an explicit decision that it needs none), and the rejected names are recorded (ANL-10).
- Every location in the designs is a source range from a start line/column to an end line/column (POS-6).
- `semantic-analysis.md` defines each analysis as an incremental query, with its fixed-point strategy, ordering, and diagnostics, including ownership checking for all four pointer kinds (ANL-11).
- The designs define the built-in comptime intrinsic set, the inference rule and its handling of cycles and indirect calls, evaluation triggers, value lifting, and dead code elimination for both modes. Mode inference and reachability are incremental queries.
- The designs specify the RVSDG entity/component model, region and port invariants, value/state dependencies and effect ordering, lowering and evaluation, and stable incremental identities distinct from ECS entity handles. They include worked examples of calls, branching, loops, and an effectful operation.
- The designs agree with each other. Terminology matches the [plan index](./index.md#terminology), and later decisions have been propagated to earlier documents.
- No files under `src/`, `test_cases/`, or `Cargo.toml` were changed by this step.
- The review log records the baseline observations, every decision, and the maintainer's approval of each document.

## Status

Not started

## Review log

*No entries yet.*
