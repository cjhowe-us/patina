# Patina Language

Patina is a modern programming language that bridges the gap between systems and
application programming through a unique multi-model approach to memory
management and execution models. It serves as a unifying language for
applications that require both high-performance and cross-platform development.
Current use cases include game engines, machine learning kernels, and
cross-platform mobile applications.

## Key Features

- **Multi-Stage Programming & Compile-Time Metaprogramming:** A core feature
  enabling code generation and execution at compile time. The compiler
  **splits the AST by stage**, compiles and runs earlier stages to generate
  code for later stages, and exposes a **compile-time API** for introspection
  and manipulation.
- **Multi-Model Programming**: Explicitly define and manage different execution
  and memory models within the same codebase, allowing both high-performance and
  high-productivity code to coexist safely.
- **Powerful Type System**: Combine static typing with expressive features like
  pattern matching, generics, and algebraic data types.
- **Memory Safety:** Compiler-enforced safety, especially in `system` regions,
  utilizing **mutable value semantics** for borrow checking. GC supported.
- **Cross-Platform Development**: Write code once and target multiple platforms
  with native performance through multi-stage programming capabilities.
- **Explicit Control**: Precise control over memory layout, lifetime management,
  and performance characteristics when needed.
- **Safe Interoperability**: Robust safety checks at region boundaries ensure
  reliable integration between different memory models.
- **Shader and ML Native**: Runs on GPUs using SPIR-V for graphics shaders and
  PTX for ML kernels.

## Who Is Patina For?

Patina is ideal for developers and teams working on:

- Performance-critical applications with complex user interfaces
- Game engines and real-time systems
- Embedded systems with resource constraints
- Cross-platform mobile applications
- Systems requiring both low-level optimization and high-level productivity

## Why Choose Patina?

Patina eliminates the need to choose between performance and productivity, or to
maintain separate codebases in different languages. By acknowledging the
fundamental trade-offs in language design and making them explicit, Patina
provides a unified solution for the full spectrum of modern application
development.

Stop struggling with FFI and multi-language builds - use a single language
designed for both systems and application programming.

## Syntax

The following examples demonstrate the Patina language syntax in an educational
progression from basic to more advanced concepts.

### Basics

Patina supports both single-line and multi-line comments. Variables are declared
using the `let` keyword, with optional type annotations. Mutable variables are
created using the `mut` modifier, and constants can be defined with the `static`
In Patina, the following are primitive value types:

- **Integers**: Signed (e.g., `i32`) and unsigned (e.g., `u64`) integer types
  with explicit bit widths.
- **Strings**: UTF-8 encoded string literals (e.g., `let s = "Hello, world!"`),
  including interpolation via `\()`.
- **Arrays**: Both fixed-size (e.g., `[5]i32`) and dynamic arrays (e.g.,
  `[]i32`).

```patina
// Comments start with double slashes
/* Block comments can span
   multiple lines */

let arbitraryNumber: i32 = 42
// Let can infer types from the assigned value
let arbitraryString = "Hello, world!"
print("The answer to life, the universe, and everything is \(arbitraryNumber)")

let mut counter: i32 = 0
counter = counter + 1

static let compileTimeInteger = 12
```

### Data Types

Patina has a rich type system including integer types with explicit bit widths,
pointer types with different ownership semantics, array types, and function
types.

Integer types can be signed (`i32`) or unsigned (`u64`). Pointer types can have
different modifiers: basic pointers (`*i32`), owned pointers (`*own i32`), raw
pointers (`*raw i32`), and mutable pointers (`*mut i32`).

Arrays can be fixed-size with a specified dimension (`[5]i32`) or dynamic
(`[]i32`). Function types describe the signature of callable objects.

```patina
let signed: i32 = -42
let unsigned: u64 = 42

let ptr: *i32
let ownPtr: *own i32
let rawPtr: *raw i32
let mutPtr: *mut i32

let fixedArray: [5]i32
let dynamicArray: []i32

let callback: fun(x: i32): i32
```

### Expressions

Patina supports a variety of expressions for different programming needs.

**Operators**: Patina supports arithmetic, comparison, and logical operators.

```patina
// Prefix operators
let negation = -a
let logicalNot = !true
let bitwiseNot = ~bitmask

// Arithmetic operators
let sum = a + b
let diff = a - b
let product = a * b
let quotient = a / b
let remainder = a % b

// Comparison operators
let isEqual = a == b
let isNotEqual = a != b
let isGreater = a > b
let isLess = a < b
let isGreaterOrEqual = a >= b
let isLessOrEqual = a <= b

// Logical operators
let logicalAnd = true && false
let logicalOr = true || false

// Postfix operators
let optionalValue = maybeNull?.member()

// Complex expression with multiple operators
let complex = !-~(a + b * c / d % e && f || g) == h
```

**String Interpolation**: String literals can include expressions using the
`\()` syntax.

```patina
let name = "Alice"
let greeting = "Hello, \(name)!"
```

**Indexing**: Array elements can be accessed using square bracket notation.

```patina
let element = array[index]
```

**Block Expressions**: Code blocks can be used as expressions, with the last
expression in the block becoming the result.

```patina
let sum = {
    let a = 10
    let b = 20
    a + b
}
```

### Control Flow

Patina supports several control flow constructs. If expressions can be used both
as statements and as expressions that return values. Case expressions provide
pattern matching capabilities with multiple condition branches. Blocks can be
labeled, and the `break` statement can target specific labeled blocks,
optionally returning a value.

```patina
let result = if (x > 0) {
    "positive"
} else if (x < 0) {
    "negative"
} else {
    "zero"
}

let description = case {
    if (age < 13) {
        "child"
    }
    if (age >= 13 && age < 20) {
        "teenager"
    }
    else {
        "adult"
    }
}

:outer {
    :inner {
        if (condition) {
            break :inner
        }
        if (emergency) {
            break :outer 42
        }
    }
}
```

### Functions

Functions are defined using the `fun` keyword, followed by the function name,
parameters with type annotations, and an optional return type. Function bodies
can contain multiple statements and control flow structures.

```patina
fun add(a: i32, b: i32): i32 {
    return a + b
}

fun max(a: i32, b: i32): i32 {
    if (a > b) {
        return a
    } else {
        return b
    }
}

fun square(x: i32): i32 {
    return x * x
}
```

**Function Calls and Method Chaining**: Functions can be called with arguments,
and methods can be chained.

```patina
let result = calculate(x, y)
let processed = data.process().filter().sort()
```

**Lambdas**: Anonymous functions can be defined inline.

```patina
let adder = fun(a: i32, b: i32): i32 {
    a + b
}
```

### Annotations

Annotations provide metadata for declarations. They can be used to mark
functions as deprecated, provide optimization hints, or even add functionality.

```patina
@deprecated
fun oldFunction() {
    return
}

@inline(always)
fun fastFunction() {
    return
}

@verify(n > 0)
fun factorial(n: i32): i32 {
    if (n <= 1) {
        return 1
    } else {
        return n * factorial(n - 1)
    }
}
```

### Factorial (Recursion)

This example demonstrates a recursive factorial function within a module. The
function checks if the input is less than or equal to 1 (base case) and returns
1, otherwise it recursively calculates n multiplied by factorial(n-1).

```patina
module Math {
    fun factorial(n: i32): i32 {
        if (n <= 1) {
            return 1
        } else {
            return n * factorial(n - 1)
        }
    }
}
```

### Fibonacci (Iteration)

This example shows an iterative implementation of the Fibonacci sequence. It
handles the base cases (0 and 1) directly, then uses a loop to calculate larger
Fibonacci numbers by iteratively updating two variables that track consecutive
numbers in the sequence.

```patina
module Math {
    fun fibonacci(n: i32): i32 {
        if (n <= 1) {
            return n
        }

        let mut a: i32 = 0
        let mut b: i32 = 1
        let mut i: i32 = 2

        while (i <= n) {
            let temp: i32 = a + b
            a = b
            b = temp
            i = i + 1
        }

        return b
    }
}
```

### Binary Search

This example implements a binary search algorithm, which efficiently finds a
target value in a sorted array. The algorithm repeatedly divides the search
range in half, comparing the middle element with the target value to determine
which half to continue searching.

```patina
module Algorithms {
    fun binarySearch(arr: []i32, target: i32): i32 {
        let mut low: i32 = 0
        let mut high: i32 = arr.length - 1

        while (low <= high) {
            let mid: i32 = low + (high - low) / 2

            if (arr[mid] == target) {
                return mid
            }

            if (arr[mid] < target) {
                low = mid + 1
            } else {
                high = mid - 1
            }
        }

        return -1
    }
}
```

## LLMs

This section describes the LLM agents used to develop the Patina language and
compiler.

### Agent Hierarchy

#### Executive Layer

- **Human Intention Agent**: Dispatches human intentions to the correct agent
- **Triage Agent**: Handles initial task triage when an issue is created
- **Coordinator Agent**: Orchestrates development in a centralized location
- **Human Intervention Agent**: Monitors agent progress, intervention decisions
- **Resource Optimization Agent**: Performs cost-benefit analysis and budgeting

#### Design Layer

- **Language Design Agents**: Design solutions to triaged issues from many
  different perspectives: Syntax, Semantic Analysis, Memory Safety, IR
  Generation
- **Task Decomposition Agent**: Breaks down tasks into subtasks
- **Architecture Agent**: Designs the system architecture in collaboration with
  a human. Uses the Gemini live API to communicate with the architecture agent
  using voice input. Involves the human much more often than the other agents.

#### Implementation Layer

- **Documentation Agent**: Reads and writes documentation
- **Code Review Agent**: Ensures code quality
- **Infrastructure Agents**: Cloud, Repository, CI/CD, Deployment
- **Quality Assurance Agents**: Unit Testing, QA/Validation

#### Meta Layer

- **Agent Development Agent**: Creates and improves agent workflows
- **Prompt Engineering Agent**: Creates and refines the prompts used by agents
- **Tool Development Agent**: Creates and improves agent tools
- **Workflow Evaluation Agent**: Analyzes processes to optimize workflows and
  estimate costs

### GitHub-Centric Workflow

1. **Task Management**
   - Issues as primary task containers
   - Task decomposition creates linked child issues
   - Pull requests deliver code changes
   - Structured communication via comments

2. **Workflow Stages**
   - Issue triage and decomposition
   - Implementation in domain-specific agents
   - Blocker resolution with strategic human intervention
   - Pull request review and integration
   - Continuous system improvement

3. **Human Intervention**
   - Selective involvement based on cost-benefit analysis
   - Precise questions to maximize input value
   - Transparent decision documentation

### Tools

- **Dev Environment Tool**: Manages isolated development environments
- **Command Execution Tool**: Executes commands in controlled contexts
- **Issue Reporter Tool**: Using MCP server for Github to report issues to
  resolve in separate threads of work.
- **Human Intervention Tool**: Loops a human in to resolve blockers, providing
  an dev instance to SSH into with Windsurf.
- **Budgeting Tool**: Tracks and updates credit availability to aid
  in budgeting and cost-benefit analysis
- **Cost Estimation Tool**: Estimates the cost of a workflow in credits based on
  the previous cost of similar tasks and workflows
- **Cost-Benefit Analysis Tool**: Performs cost-benefit analysis to determine
  the value of a task relative to the number of credits it is predicted to cost
