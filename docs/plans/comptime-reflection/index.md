# Comptime Reflection Plan

> **Base name:** `comptime-reflection` · **Status:** Active · **Supersedes:** phase-separation (removed; see [carry-over](#phase-separation-carry-over))
>
> Work on this plan with `/work-plan comptime-reflection` (see [`.github/skills/work-plan`](../../../.github/skills/work-plan/SKILL.md) and the [plan conventions](../README.md)).

## Summary

Patina is an unfinished compiler whose executable pipeline stops at parsing. This plan redirects the project toward a language in which **most features are supplied by compile-time (`comptime`) metaprogramming**, similar in spirit to C++ static reflection. Code running at compile time inspects program entities (reflection) and constructs new ones (reification). The compiler must also be **incremental**: it should not reparse unchanged files or recompute facts about unchanged items, including facts produced by reflection.

This is a **plan for a plan**. Step 1 produces design documents in `docs/designs/` and walks the maintainer through reviewing them. Step 2 turns the approved designs into concrete implementation steps. No compiler code is changed until the designs are approved.

## Goals

1. **Comptime metaprogramming as the primary extension mechanism.** Language features that would conventionally be built into the compiler, such as generics, derives, and many declaration forms, are expressed as comptime functions where practical. Whether code runs at comptime, runtime, or both is inferred from its use of built-in comptime intrinsics, never written in source.
2. **Static reflection and reification.** Comptime code can obtain handles to declarations and types, inspect their facts, and construct new items that are analyzed like handwritten code. Generated code is hygienic.
3. **Incremental compilation.** Unchanged files are not reparsed. Facts for unchanged items are not recomputed. Results are reused at the granularity of files, items, and individual facts (for example signature versus body).
4. **Incremental reflection generation.** Comptime evaluation and the items it generates participate in the same dependency tracking. Comptime code may reflect only items that are already **fully reified** (sealed), so reflection follows a reification order between items: for example, code in the second round may reflect items that were fully reified in the first. A change to a sealed item invalidates only the later results that actually depended on it. The old numbered "stage" concept is being rethought and renamed (ANL-10).
5. **A faithful front end and RVSDG IR.** The parser, AST, and a new Regionalized Value State Dependence Graph (RVSDG) IR represent exactly the program that was written, or reject it with a positioned diagnostic. The IR is stored as `hecs` entities and components; the AST is not (it is a plain owned Rust tree, see AST-9). They never panic, and they never silently substitute other values.
6. **Truthful tests and documentation.** Every fixture tests what its name says, tests assert structure rather than mere success, and documentation describes what actually exists.

## Non-goals

- Implementing the full conventional feature list (traits, associated types, generic syntax, stage polymorphism) as compiler primitives.
- A native-code backend in this plan. The IR and a comptime interpreter are in scope; target code generation may be planned later.
- User-defined surface grammar (token-level macros). This is recorded as an open question, not a commitment.

## Constraints

These constraints bind every step of this plan.

| ID | Constraint |
|---|---|
| C-1 | **No generics.** Patina has no generic type or function parameters. Comptime functions that accept and return types (types are comptime values) replace them. Existing generic syntax (`struct S<T>`, `enum`, `interface`, and `impl` parameter lists) and generic type representations (`Type::TypeParameter`, `Type::Generic`) are to be removed. |
| C-2 | **Inferred comptime/runtime.** Source code never marks code as comptime: there is no `comptime` keyword and no `@comptime` annotation, and the legacy `static` forms are retired without replacement syntax. The compiler provides a fixed set of **built-in comptime intrinsics** (for example, reflection and reification operations) that any function may call. A function that calls an intrinsic is **comptime-only** and gets a comptime version. That requirement propagates transitively to its callers. A function that reaches no intrinsic, directly or transitively, is **dual-mode**: usable at both comptime and runtime. Comptime and runtime outputs each undergo **dead code elimination**, so unused versions never reach the executable or the comptime image. Mode is a compiler-derived, per-item fact. It is not stored as syntax. |
| C-3 | **Minimal grammar change.** The grammar changes only where necessary for correctness, for retiring legacy `static` forms, for removing generics, or for an approved design. Existing accepted syntax such as `fun`, `:label`, `\(expr)` interpolation, and `@name(args)` annotations is retained unless a design explicitly changes it. No grammar is added to mark comptime code. General annotations remain ordinary syntax with no comptime meaning. |
| C-4 | **Designs before code.** No compiler code, grammar, or fixture changes are made until Step 1's designs are approved and Step 2 has produced implementation steps that cite them. |
| C-5 | **Faithful or rejected.** Any construct that is accepted must be represented faithfully; otherwise it must be rejected with a positioned diagnostic. Placeholders, `todo!`, and panics on user input are defects. |
| C-6 | **Tracked comptime effects.** Every input that comptime code consumes (files, environment, configuration, compiler facts) is a tracked dependency. Untracked effects are forbidden or make a result explicitly non-cacheable. |
| C-7 | **Preserve history.** Never create merge commits; rebase only. Preserve existing uncommitted user changes. |
| C-8 | **RVSDG-based ECS IR.** The compiler IR must be a Regionalized Value State Dependence Graph, represented using `hecs` entities and components. The IR, not the AST, is the `hecs`-backed structure: the AST is a plain owned Rust tree whose only `hecs` use is a vestigial per-node side table holding `Position` (AST-9), and no design may assume an ECS AST. Designs must specify how nodes, nested regions, ports, and value/state dependencies are represented and validated. Preserve required effect ordering through state dependencies. World ownership and graph layout are design decisions, but replacing the RVSDG with a conventional CFG IR or abandoning ECS storage is not. Allocation-local entity IDs must remain separate from stable compiler identities used by incremental queries. |
| C-9 | **Deep, incremental analysis.** Patina needs more static analysis than most languages, beyond even what Rust's compiler performs: ownership, aliasing, and mutability for the four pointer kinds; mode inference; types computed by comptime functions; checking of generated code; reification ordering; effects; and reachability in both modes (ANL-11). Each analysis is an incremental query keyed by stable identities over resolved items or the RVSDG, with a defined fixed-point strategy and diagnostics that carry source ranges (POS-6). No analysis may be a whole-program pass that must rerun after every edit, and none may be weakened to a conventional compiler's level to simplify the design. |

## Supporting files

- [`findings.md`](./findings.md): the review findings with stable IDs (`PARSE-`, `GRAM-`, `AST-`, `PRINT-`, `VIS-`, `POS-`, `ANL-`, `IR-`, `INC-`, `META-`, `TEST-`, `DOC-`) and the audit of all 100 RON parser fixtures. Every step cites these IDs.

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
faithful syntax
  → stable identity and query facts (incremental foundation)
    → minimal resolution, call graph, mode inference, and type checking as queries
      → hecs-backed RVSDG IR and comptime evaluator with tracked inputs
        → ownership, effect, and other semantic analyses over the RVSDG (C-9)
          → reflection, reification, and hygiene with generated-code reanalysis
            → incremental reflection along the reification order (ANL-10)
              → comptime and runtime dead code elimination
```

Fixture truthfulness and documentation accuracy run alongside every link.

## Work areas

Step 2 decomposes these areas into implementation steps. The finding IDs are listed so that every finding has an owner.

| Area | Scope | Findings |
|---|---|---|
| WA-1 Fixture truthfulness | Rename and rewrite fixtures so each tests what its name says. Remove duplicate and corrupted error fixtures. Add expected structure and expected diagnostics. Stop trimming input. | TEST-1…TEST-9, fixture audit |
| WA-2 Grammar and parser fidelity | Atomic tokens, keyword boundaries, float ordering, strings, grouped and type expressions, unsupported-construct diagnostics, no panics. | PARSE-*, GRAM-1…GRAM-6, GRAM-9…GRAM-13 |
| WA-3 AST, spans, printer, visitors | Annotation retention; source ranges from a start line/column to an end line/column (POS-6) with provenance; printer contract; visitor forwarding; traversal model. Remove or justify the AST's vestigial `hecs` side table so that only the IR uses `hecs`. | AST-1, AST-3, AST-4, AST-6, AST-9, PRINT-*, VIS-*, POS-* |
| WA-4 Legacy `static` retirement and generics removal | Remove the legacy `static` forms from the grammar, AST, analysis, tests, and docs, with no replacement keyword or annotation. Rewrite `static_*` fixtures to express comptime evaluation through intrinsic calls, as `comptime.md` defines. Remove generic syntax and generic type representations. | GRAM-7, GRAM-8, AST-2, ANL-6, META-6, META-7, DOC-8 |
| WA-5 Semantic pipeline and RVSDG IR | Unified IDs that replace `Arena<T>`, `Id<T>`, and the tag aliases; resolution; type checking; a call graph; lowering into a resolved RVSDG using `hecs` entities/components; graph validation; and a driver that runs beyond parsing. Graph identity and ownership must support per-item incremental reuse (C-8). | ANL-1…ANL-5, ANL-7…ANL-9, IR-* |
| WA-6 Incremental query foundations | Compiler database, stable `FileId`/`ItemId`, fingerprints, dependency recording, early cutoff, per-query diagnostics. | INC-*, AST-7, AST-8 |
| WA-7 Comptime inference, evaluator, reflection, reification | The built-in intrinsic set, comptime/runtime mode inference over the call graph, evaluation triggers and value lifting, the evaluator, reflection handles, the reification API, hygiene, generated-item identity, general annotation semantics, and dead code elimination for comptime and runtime outputs. | META-1…META-5, META-8, META-11, META-12, AST-5, GRAM-11 |
| WA-8 Incremental reflection | Replace numbered stages with item sealing and a reification order, under the name `comptime.md` chooses (ANL-10). Reflection queries that see only sealed items, incremental mode inference and reachability, and invalidation along the reification order. | INC-6, ANL-10, META-5, META-9, META-11, META-12 |
| WA-9 Docs and status | Truthful status, superseded designs, removal of nonexistent APIs from the docs. | DOC-* |
| WA-10 Semantic analyses | The analyses that C-9 requires, as incremental queries: ownership, aliasing, mutability, and lifetimes for `*own`, `*raw`, `*mut`, and `*`; checking of comptime-computed types interleaved with comptime evaluation; effect tracking through RVSDG state edges; well-formedness and hygiene checks for generated code; fixed points over call-graph strongly connected components and region dataflow; and range-carrying diagnostics. | ANL-11, POS-6 |

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

- **comptime**: code evaluated during compilation; formerly marked with `static`. Nothing in the source marks it, neither a keyword nor an annotation. The compiler infers it (see *comptime inference*). Comptime code runs before the code that uses its results (see *reification order*).
- **comptime intrinsic**: a built-in, compiler-provided operation that is only available at compile time, such as reflecting on an item or reifying a new one. Any function may call one. The intrinsic set is defined in `comptime.md`.
- **comptime inference**: the compiler-derived classification of each function's **mode**. Calling an intrinsic makes a function comptime-only, and that propagates to its callers through the call graph. Mode is a per-item incremental fact.
- **comptime-only function**: a function that reaches a comptime intrinsic, directly or transitively. Only a comptime version is generated, and runtime code cannot call it. Runtime code can use only the comptime-evaluated results it produces.
- **dual-mode function**: a function that reaches no comptime intrinsic. It is available at both comptime and runtime. Each version is emitted only if something in that mode reaches it.
- **dead code elimination (DCE)**: removal of the comptime and runtime versions that their mode's roots do not reach, so unused versions do not affect the executable binary.
- **mode**: whether a function runs at comptime, at runtime, or both. Mode is inferred per function (see *comptime inference*) and is independent of reification order.
- **stage** (provisional legacy term): the old numbered evaluation level, `analysis::Stage(u32)`, with 0 for compile time and 1 for runtime. It conflates mode, sealing, and reification order, and the name collides with compiler pipeline stages. ANL-10 replaces it, and `comptime.md` chooses the new names. Until then, this plan uses "stage" only when referring to the existing code.
- **reification order**: the partial order between items induced by reflection and reification. An item's comptime code runs only after every item it reflects on is sealed. Cycles are diagnosed. Any numbering, such as the "round" in Goal 4, is derived from this order for display only.
- **reflection**: comptime access to compiler facts about an entity through a handle, such as a type's fields or a function's signature.
- **reification**: comptime construction of new program entities (items, expressions, types) that are inserted into the program and analyzed normally.
- **fully reified** (sealed): an item whose generation has finished, so no further comptime code can change its facts. Only sealed items may be reflected, which is what makes reflection results stable and cacheable.
- **source range**: the location of a construct from a start line/column to an end line/column, covering every character in between (POS-6). It replaces the start-only `Position`.
- **query** / **fact**: a memoized, dependency-tracked computation over the program (for example, "the signature of item X"), and its result.
- **RVSDG**: Regionalized Value State Dependence Graph, the required compiler IR. It represents computation through value and state dependencies and structured control through nested regions. Its representation uses `hecs` entities and components; stable query identities do not depend on entity allocation order. The RVSDG is the only compiler structure required to use `hecs`.
- **AST**: the parser's output, a plain owned Rust tree of structs and enums in `src/ast/nodes.rs`. It is not an ECS structure. Each node currently carries a `hecs::Entity` whose only use is a side table of `Position` components (AST-9).
