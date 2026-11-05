# Comptime Reflection Plan

> **Base name:** `comptime-reflection` · **Status:** Active · **Supersedes:** phase-separation (removed; see [carry-over](#phase-separation-carry-over))
>
> Work on this plan with `/work-plan comptime-reflection` (see [`.github/skills/work-plan`](../../../.github/skills/work-plan/SKILL.md) and the [plan conventions](../README.md)).

## Summary

Patina is an unfinished compiler whose executable pipeline stops at parsing. This plan redirects the project toward a language in which **most features are supplied by compile-time (`comptime`) metaprogramming**, similar in spirit to C++ static reflection. Code running at compile time inspects program entities (reflection) and constructs new ones (reification). The compiler must also be **incremental**: it should not reparse unchanged files or recompute facts about unchanged items, including facts produced by reflection.

This is a **plan for a plan**. Step 1 produces design documents in `docs/designs/` and walks the maintainer through reviewing them. Step 2 turns the approved designs into concrete implementation steps. No compiler code is changed until the designs are approved.

### Source-checked baseline (2026-11-04)

The current source exports only `analysis`, `ast`, and `parser`; the CLI only parses (`src/lib.rs:9–17`, `src/main.rs:24–40`). **There is no IR implementation, including no RVSDG regions**, lowering, or evaluator. ECS-backed RVSDG storage is a requirement, not an implemented feature (IR-1, IR-3).

The AST's children already form an owned Rust tree (`src/ast/nodes.rs:16–20, 61–77`), but `Ast::world` and per-node entities still exist as position storage (`src/ast/mod.rs:35–44, 63–118`, `src/parser.rs:62–70, 171–186`). This leftover ECS coupling must be removed, not expanded or treated as the AST's intended architecture.

The grammar already has recursive block comments, but the parser discards comments and annotations; neither comment nesting depth nor a documentation extraction path exists. Backticks currently delimit identifiers, and `~` is currently a generic operator, not a splice node. The required language syntax is **backticks for quotation, `~` for splicing, and `@symbol` / `@symbol(args)` for annotations** (GRAM-11, GRAM-14, GRAM-16, GRAM-17). A borrow checker is also absent (ANL-12). Dependency and Pest printing observations are versioned in [findings](./findings.md#dep--dependencies).

## Goals

1. **Comptime metaprogramming as the primary extension mechanism.** Language features that would conventionally be built into the compiler, such as generics, derives, and many declaration forms, are expressed as comptime functions where practical. Whether code runs at comptime, runtime, or both is inferred from its use of built-in comptime intrinsics, never written in source.
2. **Static reflection and reification.** Comptime code can obtain handles to declarations and types, inspect their facts, and construct new items that are analyzed like handwritten code. Generated code is hygienic.
3. **Incremental compilation.** Unchanged files are not reparsed. Facts for unchanged items are not recomputed. Results are reused at the granularity of files, items, and individual facts (for example signature versus body).
4. **Incremental reflection generation.** Comptime evaluation and the items it generates participate in the same dependency tracking. Comptime code may reflect only items that are already **fully reified** (sealed), so reflection follows a reification order between items: for example, code in the second round may reflect items that were fully reified in the first. A change to a sealed item invalidates only the later results that actually depended on it. The old numbered "stage" concept is being rethought and renamed (ANL-10).
5. **A faithful front end and RVSDG IR.** The parser, AST, and a new Regionalized Value State Dependence Graph (RVSDG) IR represent exactly the program that was written, or reject it with a positioned diagnostic. The AST must remain an owned tree without ECS storage. `hecs` is for the IR and semantic analysis: analysis results may be separate components on the relevant analysis entities. RVSDG regions are implementation work, not an existing capability. Parsing and lowering must not panic on user input or silently substitute other values.
6. **Truthful tests and documentation.** Every fixture tests what its name says, tests assert structure rather than mere success, and documentation describes what actually exists.
7. **Documentation-ready syntax.** Retain all comments in the AST, including nested block comments and their nesting information, with source order, ranges, and attachment rules sufficient to generate documentation. Preserve annotations, backtick quotations, and `~` splices through parsing, traversal, and printing.
8. **Borrow checking.** Implement an incremental borrow checker for ownership, moves, loans, aliasing, mutability, and lifetimes across all four pointer kinds, with explicit raw-pointer rules, for both comptime and runtime code and reified code.
9. **Maintainable parser and dependencies.** Use Pest's source-text APIs where they suffice, design a grammar-aligned printer for changed or generated syntax, and modernize dependency requirements and the lockfile in reviewed implementation steps. Replace direct `thiserror` usage with `derive_more`.

## Non-goals

- Implementing the full conventional feature list (traits, associated types, generic syntax, stage polymorphism) as compiler primitives.
- A native-code backend in this plan. The IR and a comptime interpreter are in scope; target code generation may be planned later.
- Dead-code elimination as a correctness requirement. It may be considered later as an optimization; programs must have the same semantics without it.

## Constraints

These constraints bind every step of this plan.

| ID | Constraint |
|---|---|
| C-1 | **No generics.** Patina has no generic type or function parameters. Comptime functions that accept and return types (types are comptime values) replace them. Existing generic syntax (`struct S<T>`, `enum`, `interface`, and `impl` parameter lists) and generic type representations (`Type::TypeParameter`, `Type::Generic`) are to be removed. |
| C-2 | **Inferred comptime/runtime.** Source code never marks code as comptime: there is no `comptime` keyword and no `@comptime` annotation, and the legacy `static` forms are retired without replacement syntax. The compiler provides a fixed set of **built-in comptime intrinsics** (for example, reflection and reification operations) that any function may call. A function that calls an intrinsic is **comptime-only**. That requirement propagates transitively to its callers. A function that reaches no intrinsic, directly or transitively, is **dual-mode**: usable at both comptime and runtime. Mode is a compiler-derived, per-item fact, not syntax. |
| C-3 | **Compiler-defined syntax.** Backticks quote code; `~` splices code; `@symbol` and `@symbol(args)` annotate a target; `&own`, `&raw`, `&mut`, and bare `&` are the four pointer-type prefixes. `*` remains the multiplication operator, not a pointer prefix. These choices are fixed, not alternatives to intrinsic calls. Update conflicting quote, splice, and pointer rules and define unambiguous quotation boundaries, nesting, and escaping. Retain `fun`, `:label`, `\(expr)` interpolation, and qualified annotation names. Patina syntax is defined by its compiler. Other grammar changes are limited to correctness, retiring `static`, removing generics, or an approved design. No grammar marks comptime mode; annotations carry no comptime/runtime meaning. |
| C-4 | **Designs before code.** No compiler code, grammar, or fixture changes are made until Step 1's designs are approved and Step 2 has produced implementation steps that cite them. |
| C-5 | **Faithful or rejected.** Any construct that is accepted must be represented faithfully; otherwise it must be rejected with a positioned diagnostic. Placeholders, `todo!`, and panics on user input are defects. |
| C-6 | **Tracked comptime effects.** Every input that comptime code consumes (files, environment, configuration, compiler facts) is a tracked dependency. Untracked effects are forbidden or make a result explicitly non-cacheable. |
| C-7 | **Preserve history.** Never create merge commits; rebase only. Preserve existing uncommitted user changes. |
| C-8 | **No ECS in the AST; use `hecs` for IR and semantic analysis.** The compiler IR must be a Regionalized Value State Dependence Graph using `hecs` entities and components. The AST must be a plain owned Rust tree with ordinary Rust storage for source spans, comments, and annotations; remove its existing ECS side table and entity handles. Neither keeping that coupling nor moving AST nodes into ECS is a design option. Semantic analyses may represent each analysis result as a distinct component attached to the relevant analysis entity. Designs must specify and implementation steps must build IR operations, nested regions, ports, and value/state dependencies, including effect ordering. None of those IR structures exists yet. World ownership and graph layout are design decisions; substituting a conventional CFG IR or abandoning ECS for the IR is not. Allocation-local entity IDs must remain separate from stable compiler identities used by incremental queries. |
| C-9 | **Deep, incremental analysis, including a borrow checker.** Required analyses include ownership, borrowing, aliasing, mutability, and lifetimes for the four pointer kinds; mode inference; comptime-computed types; generated-code checking; reification ordering; and effects (ANL-11, ANL-12). Dead-code elimination is not required for correctness. Analyses are target requirements, not implemented capabilities or a measured comparison with another compiler. Each analysis is an incremental query keyed by stable identities over resolved items or the RVSDG, with a defined fixed-point strategy and source-range diagnostics (POS-6). No analysis may require a whole-program rerun after every edit. Borrow checking must cover comptime, runtime, and reified code, with an explicit safety boundary for raw pointers. |
| C-10 | **Retained comments and explicit depth.** Parse and keep line and block comments in the non-ECS AST, including leading, trailing, interstitial, nested, and file-only comments. Preserve text, kind, order, ranges, and nesting information for documentation extraction and printing. Track block-comment depth: zero outside a comment, increment on `/*`, decrement on `*/`, resume code only at zero, and diagnose EOF at nonzero depth. Define how nested comment records preserve that depth and how excessive nesting is diagnosed without a panic. Comment delimiters inside string literals are not comments. Documentation classification and attachment are design decisions; dropping ordinary comments is not. |

## Supporting files

- [`findings.md`](./findings.md): the source-checked findings with stable IDs (`PARSE-`, `GRAM-`, `AST-`, `PRINT-`, `VIS-`, `POS-`, `ANL-`, `IR-`, `INC-`, `META-`, `TEST-`, `DOC-`, `DEP-`), versioned Pest/dependency research, and the audit of all 100 RON parser fixtures. Every step cites these IDs.

## Status

| Step | Title | Status | Review gate | Prerequisites |
|---|---|---|---|---|
| [1](./step-1.md) | Design documents and guided review | Not started | **Yes** | none |
| [2](./step-2.md) | Implementation planning | Not started | **Yes** | Step 1 |
| 3+ | *Created by Step 2* | — | — | Step 2 |

Steps 3 and later are intentionally unwritten. Step 2 creates them from the approved designs, ordered by the dependency chain below.

## Dependency chain

The implementation steps that Step 2 produces must respect this order. Each link depends on the ones before it:

```text
dependency baseline and upgrades + truthful fixture harness
  → faithful syntax, retained comments, and grammar-aligned printing
    → stable identity and query facts (incremental foundation)
      → minimal resolution, call graph, mode inference, and type checking as queries
        → hecs-backed RVSDG IR, including region construction and validation
          → borrow checker, effects, and a comptime evaluator with tracked inputs
            → reflection, reification, and hygiene with generated-code reanalysis
              → incremental reflection along the reification order (ANL-10)
```

Fixture truthfulness and documentation accuracy run alongside every link. Dependency upgrades and harness work can proceed as independent early slices after design approval; grammar and printer decisions must be checked on the selected Pest version. Documentation extraction depends on retained comments and their attachment rules. This orders enabling implementation slices, not a single-pass pipeline: typing and evaluation of comptime-computed types must interleave. Do not expose comptime evaluation or generated-code execution before the applicable borrow/effect checks run. Optional dead-code elimination may be added after semantic correctness and must preserve observable program behavior.

## Work areas

Step 2 decomposes these areas into implementation steps. The finding IDs are listed so that every finding has an owner.

| Area | Scope | Findings |
|---|---|---|
| WA-1 Fixture truthfulness | Rename and rewrite fixtures so each tests what its name says. Remove duplicate and corrupted error fixtures. Add expected structure and expected diagnostics. Stop trimming input. Cover syntax, comment depth/retention, documentation extraction, printer round trips, regions, and borrow checking. | TEST-1…TEST-11, fixture audit |
| WA-2 Grammar and parser fidelity | Atomic tokens, keyword boundaries, float ordering, strings, grouped and type expressions, backtick quoting, `~` splicing, `@symbol` / `@symbol(args)` annotations, retained comments with explicit nested-block depth, unsupported-construct diagnostics, no panics. | PARSE-*, GRAM-1…GRAM-6, GRAM-9…GRAM-17 |
| WA-3 AST, spans, printer, visitors | Non-ECS comment and annotation storage; source ranges and provenance; quote/splice nodes; documentation extraction and attachment; Pest source replay versus grammar-aligned emission for generated/changed ASTs; traversal model. Remove the AST's ECS side table and handles, not the owned tree. | AST-1, AST-3, AST-4, AST-6, AST-10, PRINT-*, VIS-*, POS-* |
| WA-4 Legacy `static` retirement and generics removal | Remove the legacy `static` forms from the grammar, AST, analysis, tests, and docs, with no replacement keyword or annotation. Rewrite `static_*` fixtures to express comptime evaluation through intrinsic calls, as `comptime.md` defines. Remove generic syntax and generic type representations. | GRAM-7, GRAM-8, AST-2, ANL-6, META-6, META-7, DOC-8 |
| WA-5 Semantic pipeline and RVSDG IR | Unified IDs that replace `Arena<T>`, `Id<T>`, and the tag aliases; resolution; type checking; a call graph; lowering into a resolved RVSDG using `hecs` entities/components; explicit region construction, region arguments/results and ports, value/state edges, graph validation; and a driver that runs beyond parsing. Semantic-analysis results may be stored as distinct `hecs` components on their relevant entities. All IR work is new. Graph identity and ownership must support per-item incremental reuse (C-8). | ANL-1…ANL-5, ANL-7…ANL-9, IR-* |
| WA-6 Incremental query foundations | Compiler database, stable `FileId`/`ItemId`, fingerprints, dependency recording, early cutoff, per-query diagnostics. | INC-*, AST-7, AST-8 |
| WA-7 Comptime inference, evaluator, reflection, reification | The built-in intrinsic set, comptime/runtime mode inference over the call graph, evaluation triggers and value lifting, the evaluator, reflection handles, the reification API, hygiene, generated-item identity, and general annotation semantics. Syntax is compiler-defined. | META-1…META-5, META-8, META-11, AST-5, GRAM-11 |
| WA-8 Incremental reflection | Replace numbered stages with item sealing and a reification order, under the name `comptime.md` chooses (ANL-10). Reflection queries that see only sealed items, incremental mode inference, and invalidation along the reification order. | INC-6, ANL-10, META-5, META-9, META-11 |
| WA-9 Docs and status | Truthful status, superseded designs, removal of nonexistent APIs from the docs. | DOC-* |
| WA-10 Semantic analyses and borrow checker | The analyses that C-9 requires, as incremental queries: an explicit borrow checker with move/initialization state, loans, aliasing, mutability, lifetimes, drops, and pointer-kind conversions for `&own`, `&raw`, `&mut`, and `&`; comptime-computed types interleaved with evaluation; effects through RVSDG state edges; generated-code checks; fixed points over call-graph strongly connected components and RVSDG region dataflow; and range-carrying diagnostics. | ANL-11, ANL-12, POS-6 |
| WA-11 Dependencies | Audit manifest requirements against locked and available releases; upgrade compatible and breaking dependency lines with explicit API/MSRV and fixture-format migration plans; keep Pest and its derive/generator stack compatible; remove unused dependencies where justified; commit manifest and lockfile changes together in the later implementation step. | DEP-1, DEP-2, PRINT-3 |

## Phase-separation carry-over

The superseded phase-separation plan was assessed against the code on 2026-11-04 and has been removed along with the old stage-levels design and conflicting RFCs. This table preserves its assessment and unfinished work; the [removal record](./findings.md#removed-documentation) records the other dispositions. Deleted documents are not prerequisites for this plan.

| Phase-separation step | Assessed status | Implemented | Remaining | Disposition here |
|---|---|---|---|---|
| 1 Foundation | Partial | `analysis::Stage(u32)` with `COMPILE_TIME`/`RUNTIME`, `next`/`prev`, `can_see_symbol_at`, and `StageError` (`src/analysis/stage.rs`). An ECS `Stage` component (`src/ast/components.rs:33–35`) duplicates it. A stage-aware `SymbolTable` (`declare_staged`, `lookup_at_stage`, `lookup_all_visible_stages`). A stage scope in `AnalysisContext`. | `@[stage(n)]` parsing (annotations are dropped, PARSE-8). The `StageAware`/`StageAnalyzer`/`StageSeparable` traits. Quote/splice markers. Cross-stage reference flags. | Covered by `comptime.md` (separating mode, sealing, and reification order from numbered stages and renaming the concept, ANL-10; comptime/runtime inference; removing the duplicate `Stage` types, AST-6), `syntax-and-parsing.md` (general annotation retention), and `incremental-reflection.md` (reification order). The `@[stage(n)]` proposal is **not adopted**. Modes and ordering are inferred from intrinsic use and from reflection and reification dependencies, not written in source. |
| 2 Stage analysis | Not started | — | A `StageCalculator` over the AST. | Becomes per-item mode inference and reification-order queries over the call graph in `comptime.md` and `incremental-compilation.md`, not a whole-AST pass. |
| 3 Stage separation | Not started | — | Split the unified AST into per-stage ASTs. | **Replaced.** Duplicated per-stage ASTs are not required. Each item is analyzed through queries (`compiler-architecture.md`, `incremental-compilation.md`). Revisit only if a design requires it. |
| 4 Stage-0 evaluation | Not started | — | A stage-0 evaluator that generates stage-1 AST. Compile-time file I/O has no tracked-input contract. | Covered by `comptime.md` and `static-reflection.md`. All comptime I/O becomes a tracked input (C-6, INC-6). |
| 5 Type system | Partial | `Type::Code { inner, from_stage }` (`src/analysis/types.rs:60`), `StagedType` (`types.rs:94`), and `check_splice`/`check_quote` (`types.rs:332–353`). None are connected to any pass. | Stage-aware type checking of real programs. Stage polymorphism. | Type checking is covered by `ast-and-ir.md`, `semantic-analysis.md`, and `compiler-architecture.md`. `Type::TypeParameter`/`Type::Generic` are removed (C-1). `Type::Code { from_stage }` and `StagedType` are revisited under ANL-10. **Stage polymorphism is dropped** because it conflicts with C-1; comptime functions that take a type cover the use cases. |
| 6 Integration | Not started | — | An end-to-end pipeline and tests. `src/main.rs` only parses. | Replaced by continuous vertical slices defined in Step 2, each with end-to-end and incremental-equivalence tests (`testing-strategy.md`). |

None of the phase-separation acceptance checkboxes were checked when it was superseded.

## Terminology

Unless explicitly labeled current or legacy, these terms describe the target design, not implemented behavior.

- **comptime**: code evaluated during compilation; formerly marked with `static`. Nothing in the source marks it, neither a keyword nor an annotation. The compiler infers it (see *comptime inference*). Comptime code runs before the code that uses its results (see *reification order*).
- **comptime intrinsic**: a built-in, compiler-provided operation that is only available at compile time, such as reflecting on an item or reifying a new one. Any function may call one. The intrinsic set is defined in `comptime.md`.
- **comptime inference**: the compiler-derived classification of each function's **mode**. Calling an intrinsic makes a function comptime-only, and that propagates to its callers through the call graph. Mode is a per-item incremental fact.
- **comptime-only function**: a function that reaches a comptime intrinsic, directly or transitively. Only a comptime version is generated, and runtime code cannot call it. Runtime code can use only the comptime-evaluated results it produces.
- **dual-mode function**: a function that reaches no comptime intrinsic. It is available at both comptime and runtime. Each version is emitted only if something in that mode reaches it.
- **dead code elimination (DCE)**: an optional optimization that removes unreachable comptime/runtime work. It is not required for semantic correctness; an implementation without DCE must preserve the same observable behavior.
- **mode**: whether a function runs at comptime, at runtime, or both. Mode is inferred per function (see *comptime inference*) and is independent of reification order.
- **stage** (provisional legacy term): the old numbered evaluation level, `analysis::Stage(u32)`, with 0 for compile time and 1 for runtime. It conflates mode, sealing, and reification order, and the name collides with compiler pipeline stages. ANL-10 replaces it, and `comptime.md` chooses the new names. Until then, this plan uses "stage" only when referring to the existing code.
- **reification order**: the partial order between items induced by reflection and reification. An item's comptime code runs only after every item it reflects on is sealed. Cycles are diagnosed. Any numbering, such as the "round" in Goal 4, is derived from this order for display only.
- **reflection**: comptime access to compiler facts about an entity through a handle, such as a type's fields or a function's signature.
- **reification**: comptime construction of new program entities (items, expressions, types) that are inserted into the program and analyzed normally.
- **fully reified** (sealed): an item whose generation has finished, so no further comptime code can change its facts. Only sealed items may be reflected, which is what makes reflection results stable and cacheable.
- **source range**: the location of a construct from a start line/column to an end line/column, covering every character in between (POS-6). It extends the existing `Position` byte-range model, whose line/column information currently describes only the start.
- **query** / **fact**: a memoized, dependency-tracked computation over the program (for example, "the signature of item X"), and its result.
- **RVSDG**: Regionalized Value State Dependence Graph, the required, not-yet-implemented compiler IR. It represents computation through value and state dependencies and structured control through nested regions. Its representation uses `hecs` entities and components; stable query identities do not depend on entity allocation order.
- **RVSDG region**: a graph region owned by a structural node, with explicit arguments, results, and contained operations. No such representation exists yet (IR-3). It is distinct from a lifetime/borrow region used by the borrow checker.
- **AST**: the parser's owned tree of structs and enums in `src/ast/nodes.rs`. The AST must not use ECS, even for metadata. Existing entity fields and the position side table are migration work under C-8, not a reason to redesign the tree as ECS.
- **quotation / splice / annotation**: backticks construct quoted code; `~` inserts code through splicing; `@symbol` or `@symbol(args)` attaches an annotation. `&own`, `&raw`, `&mut`, and bare `&` are the pointer-type prefixes; `*` remains multiplication. None is a source-level comptime-mode marker. Designs settle quotation nesting and splice binding, not these token choices.
- **comment depth**: the count of unmatched block-comment openers during parsing, zero outside comments. AST nesting/depth records must preserve the information required by C-10.
- **borrow checker**: the planned incremental analysis of moves, loans, access conflicts, and lifetime/escape constraints. Defining pointer modifiers or relying on Rust's own borrow checker is not an implementation of Patina's borrow checker.
