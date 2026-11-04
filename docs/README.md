# Patina Documentation

## Compiler Architecture

Patina's AST is a plain owned Rust tree. It carries a vestigial `hecs` side table that holds only node positions, and it is not an Entity-Component-System structure (finding AST-9). The project is being redirected toward compile-time (`comptime`) metaprogramming and static reflection, with incremental compilation:
- Plain owned AST, parsed with Pest
- Semantic analysis infrastructure (not yet integrated)
- Planned `comptime` metaprogramming and static reflection, replacing generics
- Planned incremental compilation and incremental reflection
- Required RVSDG intermediate representation using `hecs` entities and components; the IR is the only structure required to use `hecs`

> The active plan is [comptime-reflection](./plans/comptime-reflection/index.md). Several claims in the developer and status docs are under review; see [findings](./plans/comptime-reflection/findings.md).

## Documentation Structure

### 📘 User Documentation
**For Patina Language Users**
- User documentation has not been written yet. To build and run the compiler, see the developer [Getting Started](./developer/getting-started.md) guide.

### 👷 [Developer Documentation](./developer/)
**For Compiler Contributors**
- [**Architecture**](./developer/architecture/) - Compiler design
  - [Overview](./developer/architecture/overview.md) - Current pipeline and component status
  - [Visitor Pattern](./developer/architecture/visitor-pattern.md) - AST traversal patterns
- [**AST**](./developer/ast.md) - Node tree and its `hecs` position side table
- [**Getting Started**](./developer/getting-started.md) - Development setup
- [**Testing**](./developer/testing.md) - Test infrastructure

### 📋 [Plans](./plans/README.md)
**Implementation Roadmaps**
- [**Plans README**](./plans/README.md) - Plan conventions, step format, and plan index
- [**Comptime Reflection**](./plans/comptime-reflection/index.md) - **Active**: comptime metaprogramming, static reflection, and incremental compilation
  - [Findings](./plans/comptime-reflection/findings.md) - Review findings and fixture audit
  - [Step 1](./plans/comptime-reflection/step-1.md) - Design documents and review
  - [Step 2](./plans/comptime-reflection/step-2.md) - Implementation planning
  - Work on a step with the [`work-plan` skill](../.github/skills/work-plan/SKILL.md): `/work-plan comptime-reflection`

### 🎨 Designs
**Architectural Designs**
- [Step 1](./plans/comptime-reflection/step-1.md) will create and review design documents in `docs/designs/`.

### 📝 [RFCs](./rfcs/)
**Problem Statements**

These remaining RFCs cover concerns that still apply. Their examples and implementation suggestions are not approved designs; the [active plan](./plans/comptime-reflection/index.md) governs scope and ordering. Conflicting RFCs were removed; see the [removal record](./plans/comptime-reflection/findings.md#removed-documentation).

- [**Error Handling**](./rfcs/error-handling.md) - Robustness issues
- [**Documentation**](./rfcs/documentation.md) - Documentation gaps
- [**Testing**](./rfcs/testing.md) - Test coverage gaps
- [**Developer Experience**](./rfcs/developer-experience.md) - Tooling needs

### 📊 [Status](./status/)
**Project Status**
- [**Current Status**](./status/current.md) - Implementation progress (*claims under review*)

## Core Concepts

### Comptime Reflection
The [active plan's terminology](./plans/comptime-reflection/index.md#terminology) defines inferred comptime/runtime modes, sealing, reification order, reflection, reification, source ranges, incremental queries, and the RVSDG IR. The legacy "stage" concept is being split and renamed (finding ANL-10), and the detailed graph representation will be settled during design review.

### Key Benefits
- Clear phase boundaries
- Type-safe metaprogramming
- Optimized compilation
- Extensible architecture

## Navigation by Task

### Using Patina
→ [Getting Started](./developer/getting-started.md) (user documentation is not written yet)

### Contributing
1. [Architecture Overview](./developer/architecture/overview.md) - Understand the design
2. [AST](./developer/ast.md) - Learn the AST
3. [Comptime Reflection Plan](./plans/comptime-reflection/index.md) - Active work and its status
4. [Current Status](./status/current.md) - Implementation progress (claims under review)
5. [RFCs](./rfcs/) - Identify problems to solve

### Understanding Design
- [Comptime Reflection Plan](./plans/comptime-reflection/index.md) - Active direction
- [Design Documents and Guided Review](./plans/comptime-reflection/step-1.md) - Design scope and review process
- [Phase-Separation Carry-over](./plans/comptime-reflection/index.md#phase-separation-carry-over) - Unfinished work retained from the removed plan

### Implementing Features
1. Check [RFCs](./rfcs/) for problem definition
2. Review [Plans](./plans/README.md) for approach
3. Follow [Architecture](./developer/architecture/) patterns