# Compilation Stage Calculation Design

The purpose of this document is to outline the design for calculating compilation stages for all definitions in the Patina language.

## Overview

In the Patina language, the compilation stage of a definition determines when that definition is evaluated during compilation. This allows for better optimization and execution planning. The compilation stage is calculated based on dependency relationships between definitions.

## Data Structures

We'll use the existing data structures defined in `scope.go`:

- `Stage` (uint64): Represents the compilation stage of a definition.
- `Definition`: Contains information about a named entity, including its Stage and Usages.
- `Scope`: A hierarchical container of definitions.

## Constants

We'll define the following constants:
- `StageRuntime`: The maximum value of uint64, representing the runtime stage.
- `StageStatic`: Value of 1, representing the static compilation stage.

## Algorithm

The algorithm for calculating compilation stages follows these steps:

1. **First Pass**: Collect all function and variable definitions in the AST.
2. **Second Pass**:
   - Collect the use-def chains of each variable definition.
   - Collect every function call interdependency.

3. **Preparation**:
   - Create two sets: `activeSet` (initially empty) and `inactiveSet` (contains all definitions).
   - Sort definitions in topological order based on their dependencies to optimize the calculation.

4. **Calculation Loop**:
   - Iterate through definitions in the `inactiveSet`.
   - For each definition:
     - Check if any of its usages are in the `inactiveSet`. If so, skip it for this iteration.
     - If all usages are in the `activeSet`, calculate its compilation stage:
       - Find the maximum stage of all its usages
       - Add 1 to that value
       - Assign the result as the compilation stage of this definition
       - Move the definition to the `activeSet`
     - If it has no usages and is static, assign stage 1 and move to `activeSet`
     - If it has no usages and is not static, assign the runtime stage and move to `activeSet`
   - Repeat until the `inactiveSet` is empty.

5. **Function Call Graph Processing**:
   - Process function call dependencies in the same manner, ensuring stages are calculated correctly.

## Implementation Approach

1. Create a new `stage.go` file to implement the compilation stage calculation algorithm.
2. Define the necessary constants and functions.
3. Implement a function to calculate stages for all definitions in a given scope hierarchy.
4. Integrate with the existing grammar and AST in `parser.go`.
5. Write unit tests to verify the implementation.

## Optimization

We'll optimize the algorithm by sorting definitions in topological order based on their dependencies, which will reduce the number of passes needed to calculate all stages correctly.

## Scope Discovery

To properly test and integrate the compilation stage calculation, we need a way to discover scopes from the parsed AST:

1. Create a function to traverse the AST and build the scope hierarchy.
2. Map AST nodes to their corresponding scopes and definitions.
3. Establish use-def chains between definitions based on references in the AST.

The scope discovery process will:
- Create a new scope for each module and function body.
- Register definitions for let statements and function declarations.
- Track usage relationships between definitions based on identifier references.

## Integration

This functionality will be integrated with the existing code:
- The calculation happens after parsing and scope creation.
- It will use the existing AST nodes and Scope/Definition structures.
- It will populate the Stage field of each Definition object.

## Testing Plan

We'll implement several levels of testing:

1. **Unit Tests**:
   - Test stage calculation with manually constructed scope hierarchies and definition relationships.
   - Verify edge cases like cyclic dependencies and empty scopes.

2. **Integration Tests**:
   - Parse sample Patina code to generate ASTs.
   - Discover scopes and definitions from these ASTs.
   - Calculate compilation stages for all definitions.
   - Verify that stages are correctly assigned based on dependency relationships.

3. **Test Cases**:
   - Simple variable definitions with no dependencies
   - Variable chains (a -> b -> c)
   - Branching dependencies (multiple definitions depending on one)
   - Cyclic references (handled via the cycle-breaking logic)
   - Static vs non-static definitions
   - Function call dependencies
   - Cross-scope references

All tests will be implemented using table-driven tests with go-snaps for
snapshot testing when appropriate.
