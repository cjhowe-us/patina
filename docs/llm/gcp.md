# Patina Language - AI Agent Development Plan v1.2

## 1. Project Goal & Problem Statement

* **Goal:** Develop the compiler and core tooling for the Patina programming
  language using a team of specialized autonomous AI agents, orchestrated on
  Google Cloud Platform (GCP) and utilizing the Agent Development Kit (ADK).
* **Problem Statement:** Modern software development often bridges systems
  programming (performance) and application programming (productivity).
  This typically requires multiple languages, leading to complex FFI, build
  systems, and boundary safety issues. Patina aims to solve this with a
  single language offering a multi-model approach to memory/execution,
  allowing high-performance and high-productivity code to coexist safely.

## 2. Core Language Pillars

* **Multi-Model Programming:** Explicit definition and management of different
  execution/memory models (e.g., `system`, `managed`) in one codebase.
* **Powerful Type System:** Static typing with pattern matching, generics,
  ADTs.
* **Memory Safety:** Compiler-enforced safety, especially in `system` regions,
  utilizing **mutable value semantics** for borrow checking.
* **Multi-Stage Programming & Compile-Time Metaprogramming:** A core feature
  enabling code generation and execution at compile time. The compiler
  **splits the AST by stage**, compiles and runs earlier stages to generate
  code for later stages, and exposes a **compile-time API** for introspection
  and manipulation.
* **Explicit Control:** Precise control over memory layout, lifetimes, and
  performance when needed (`system` model).
* **Safe Interoperability:** Robust static checks at model boundaries for
  reliable integration.

## 3. Initial Repository State

All of the information in this section only applies to the AST package

The repository currently includes an AST package implemented in Go that parses
Patina source code, along with a visitor that can walk the entire Patina AST.
The parser may be missing important code constructs that still need to be added.
Use this as a starting point. The LLM code should be implemented in the llm
folder..

## Testing Infrastructure

The repository uses snapshot testing with the go-snaps library:

- `ast/test_shared.go`: Common testing utilities
- `ast/parser_test.go`: Tests for the parser
- `ast/walk_test.go`: Tests for tree traversal

Test cases and snapshots are organized in:
- `ast/testing/cases/`: JSON files containing test cases
- `ast/testing/snapshots/`: Snapshot files for test results

## Development Guidelines

When making changes to the Patina codebase:

1. **Parser Modifications**:
   - The parser is built using the participle library
   - Modify grammar definitions in the corresponding Go structs
   - Update `parser.go` for lexer rules and token mappings

2. **AST Structure Changes**:
   - Implement the appropriate interfaces (`SyntaxNode`, `Statement`, `Expression`, etc.)
   - Add necessary Position() and Children() methods for tree traversal
   - Connect new nodes to existing structures

3. **Testing**:
   - Add test cases in the appropriate JSON files in `ast/testing/cases/`
   - Run tests with `go test ./...`
   - The snapshots are stored in `ast/testing/snapshots/`
   - Update snapshots using `UPDATE_SNAPS=true go test ./...`
   - Clean up obsolete snapshots with `UPDATE_SNAPS=clean go test ./...`

4. **Best Practices**:
   - Use a declarative, functional programming style
   - Avoid mutable state where possible
   - Do not add implementation comments - only provide public documentation
     comments
   - Abstract logic into interfaces when operating on multiple data types
   - Use generics instead of interfaces for homogeneous collections

## File Modification Strategy

When implementing new features or fixing bugs:

1. First understand the corresponding AST structures
2. Modify or add struct definitions with proper participle parser tags
3. Update lexer rules if new token types are needed
4. Review the implementation to ensure all functionality works as expected
5. Add test cases to verify the changes
6. Run tests and update snapshots

## Glossary

- **AST**: Abstract Syntax Tree, a hierarchical structure representing the source code
- **Atom**: A single unit of an expression (e.g. identifier, literal, etc.)
- **Expression**: A unit of code that returns a value
- **Statement**: A unit of code that performs an action (no return value)
- **SyntaxNode**: An interface implemented by all AST nodes
- **Type**: A representation of a type (e.g. integer, pointer, array, etc.)

## 3. Technical Architecture & Requirements

* **Development Languages:** Go for infrastructure and compiler, Python for LLM
  agents.
* **Infrastructure as Code (IaC) Framework**: CDK for Terraform (Go).
* **LLM Agent Framework:** Agent Development Kit (ADK) (Python).
* **Cloud Platform:** Google Cloud Platform (GCP).
* **Agent Orchestration/Deployment:** Vertex AI (Agent Engine).
* **Source Code Management:** GitHub (`https://github.com/cjhowedev/patina`).
* **CI/CD:** GitHub Actions.
* **Artifact/State Storage:** Cloud Storage (code, ASTs, IR, binaries, state).
* **Monitoring & Logging:** Cloud Monitoring, Cloud Logging.
* **Notification System:** Cloud Pub/Sub, Cloud Functions (for agent halts).
* **Secrets Management:** GCP Secret Manager (for GitHub tokens).
* **Human Intervention:** Required for key decisions, approvals, resolving
  blocks. Notified via GitHub Issues.

## 4. Agent Roles & Responsibilities

* **Coordinator Agent:** Oversees development, decomposes tasks (including
  staged compilation steps), monitors progress, handles cross-agent decisions,
  initiates human intervention.
* **Syntax Agent:** Defines/updates formal grammar using **PEG**. Generates and
  maintains parser code. Ensures syntax supports multi-model constructs,
  **multi-stage code definition, and compile-time API calls**.
* **Compiler Pass Agent(s) - Specialized:**
  * *Semantic Analysis (Intra-Model):* Type checking, symbol tables, name
    resolution within a single model and **handling staged ASTs**.
  * *Memory Safety (System Model):* Implements **mutable value semantics for
    borrow checking** within the `system` model.
  * *Memory Management (Managed Model):* Implements strategy (ARC, GC)
    for `managed` models.
  * *Inter-Model Analysis:* **CRITICAL.** Enforces static analysis rules
    between models, verifying boundary safety.
  * *Multi-Stage Analysis:* Manages the **staged compilation process**:
    identifying code stages, overseeing **compile-time code execution**,
    handling **AST generation/integration** from earlier stages, and resolving
    calls to the **compile-time API**.
* **IR Agent:** Designs, generates, modifies Intermediate Representation(s).
  Ensures IR represents multi-model nature and **supports staged execution
  representation and dynamically generated code fragments**.
* **Optimization Agent(s):** Implements standard, model-aware, and potentially
  stage-aware optimizations.
* **Code Generation Agent(s):** Translates optimized IR to target code (LLVM
  IR, WASM, etc.). Handles backend specifics for platforms, models, and
  **different compilation stages**.
* **Testing Agent:** *Generates* unit, integration, potentially property/fuzz
  tests. Creates test cases for models, safety rules (incl. mutable value
  semantics), inter-model checks, **compile-time API usage, and staged
  compilation scenarios**. Integrates with CI for compiler code testing. May
  generate Patina code examples for the QA/Validation agent.
* **QA/Validation Agent (New):** Acts as an early *user* of the developing
  compiler. Takes Patina source examples. Attempts to **build** examples using
  the current integrated compiler state, specifically exercising the
  **multi-stage compile-run-compile cycle**. If build succeeds, attempts to
  **run** the output. Reports build/run status to Coordinator. Requires access
  to latest build artifacts.

## 5. High-Level Implementation Plan (Phased)

* **Phase 1: Foundation & Infrastructure Setup**
  * Define Patina Spec v0.1 (core multi-model syntax via **PEG**, initial
    models, interaction, basic safety goals using mutable value semantics,
    initial multi-stage concepts & compile-time API ideas).
  * Set up GCP, GitHub repo, CI/CD.
  * Configure ADK, define agent roles/tools, integrate with Vertex AI.
* **Phase 2: Monitoring & Intervention Framework**
  * Define progress metrics, halting conditions.
  * Implement agent self-monitoring/reporting.
  * Implement notification system (Agent Halt -> Pub/Sub -> Cloud Func ->
    GitHub Issue -> Human).
  * Implement Coordinator decision logic and escalation.
* **Phase 3: Core Multi-Model & Safety Implementation**
  * P1: Basic **PEG** parsing for models & initial IR structure (Syntax, IR
    Agents).
  * P2: Basic semantic analysis within one model (Semantics Agent).
  * P3: Foundational `system` model memory safety using **mutable value
    semantics** (Memory Safety Agent).
  * P4: Initial simple inter-model safety checks (Inter-Model Analysis
    Agent - High Human Oversight).
  * Testing Agent develops tests concurrently.
  * **QA/Validation Agent starts basic (single-stage) builds**.
  * Workflow: Feature Branch -> Commit -> PR -> Human Review -> Merge ->
    CI/CD (incl. QA checks).
* **Phase 4: Multi-Stage Implementation & Type System Expansion**
  * Implement **AST splitting based on stage**. (Syntax, IR, Analysis Agents)
  * Develop the **compile-time API** and mechanisms for compile-time code
    execution. (Multi-Stage Analysis Agent)
  * Implement the **compile-run-compile cycle** for staged code generation.
    (Coordinator, Multi-Stage Analysis, Code Gen Agents)
  * Add type system features (generics, ADTs) ensuring they work across
    models and stages. (Syntax, Semantics Agents).
  * Refine inter-model checks (Inter-Model Analysis Agent).
  * Add `system` model "Explicit Control" features.
  * **QA/Validation Agent focuses heavily on multi-stage build/run tests.**
  * Testing Agent generates specific multi-stage / compile-time API tests.
* **Phase 5: Optimization, Backend, Tooling & Refinement**
  * Implement standard and potentially model/stage-aware optimizations
    (Optimization Agent).
  * Refine code generation for target platforms (Code Generation Agent).
  * Develop supporting tools (LSP, debugger aware of stages) (Agents/Humans).
  * **QA/Validation Agent performs broader regression/validation testing.**
  * Continuous human oversight for architecture, safety, performance, releases.

## 6. Monitoring & Human Intervention Protocol

* **Agent Monitoring:** Agents track progress metrics, report status to
  Coordinator/Cloud Monitoring. QA Agent reports build/run success rates,
  including multi-stage scenarios.
* **Halting Conditions:** Agent halts and notifies if:
  * Progress below threshold for sustained period.
  * Requires decision outside its capabilities or faces ambiguity (e.g., in
    compile-time API behavior).
  * Needs human approval for privileged action.
  * QA Agent detects persistent build/run failures (regressions), especially
    in the multi-stage process.
* **Notification Workflow:**
  1. Agent halts.
  2. Agent sends structured message to Pub/Sub (state, task, halt reason).
  3. Cloud Function processes message.
  4. Cloud Function logs details to Cloud Logging.
  5. Cloud Function creates GitHub Issue, tagging humans with context.
  6. Optional: Further alerts (email/chat).
* **Coordinator Role:** Attempts local issue resolution. Uses QA feedback for
  bug assignment (incl. multi-stage bugs). Escalates to human intervention if
  unable to resolve, issue is widespread, or privileged action requested.
* **Human Role:** Monitor GitHub Issues, provide decisions, clarify specs
  (esp. for mutable value semantics, compile-time API), resolve blockers,
  review/approve code, analyze QA failures, perform privileged actions.
