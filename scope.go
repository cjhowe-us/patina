package patina

import "fmt"

type (
	Stage uint64
	// A scope is a named, hierarchical container of definitions.
	Scope struct {
		Name     string // The name of the scope
		IsStatic bool   // Whether the scope is static

		Definitions map[string]*Definition // Definitions in the scope by name
		Children    map[string]*Scope      // Child scopes by name
		Parent      *Scope                 // Parent scope
	}
	// A definition is a named entity in a scope.
	Definition struct {
		Name     string        // The name of the definition
		IsStatic bool          // Whether the definition is static
		Stage    Stage         // Compilation stage of the definition
		Parent   *Scope        // Parent scope of the definition
		Usages   []*Definition // The definitions this definition depends on (dependencies)
		Node     *ParseNode    // The AST node that created this definition
	}
	// A context contains the root scope of an execution environment.
	Context struct {
		Root *Scope // The root scope
	}
)

// NewScope creates and returns a new scope.
func NewScope(name string, parent *Scope, isStatic bool) *Scope {
	return &Scope{
		Name:        name,
		IsStatic:    isStatic,
		Parent:      parent,
		Definitions: make(map[string]*Definition),
		Children:    make(map[string]*Scope),
	}
}

// collectAllDefinitions recursively gathers all definitions from a scope and its children.
func collectAllDefinitions(scope *Scope) []*Definition {
	var defs []*Definition

	// Collect definitions from the current scope
	for _, def := range scope.Definitions {
		defs = append(defs, def)
	}

	// Recursively collect definitions from child scopes
	for _, childScope := range scope.Children {
		defs = append(defs, collectAllDefinitions(childScope)...)
	}

	return defs
}

// CalculateStages performs a topological sort on the definitions within the context
// to determine compilation stages and detect cyclic dependencies.
func CalculateStages(ctx *Context) error {
	definitions := collectAllDefinitions(ctx.Root)
	if len(definitions) == 0 {
		return nil // Nothing to do
	}

	// Build dependency graph (dependents map) and calculate in-degrees
	inDegree := make(map[*Definition]int)
	dependents := make(map[*Definition][]*Definition)

	for _, def := range definitions {
		// Initialize in-degree based on the number of usages (dependencies)
		inDegree[def] = len(def.Usages)
		// Initialize dependents list for this node
		dependents[def] = []*Definition{}
	}

	// Populate the dependents map (reverse graph)
	for _, defUser := range definitions {
		for _, defUsed := range defUser.Usages {
			// If defUser uses defUsed, then defUser is a dependent of defUsed
			if _, ok := dependents[defUsed]; ok { // Ensure the used definition is part of our collected set
				dependents[defUsed] = append(dependents[defUsed], defUser)
			}
		}
	}

	// Initialize queue with nodes having in-degree 0
	queue := []*Definition{}
	for _, def := range definitions {
		def.Stage = 0 // Initialize all stages to 0
		if inDegree[def] == 0 {
			queue = append(queue, def)
		}
	}

	// Process queue (Kahn's algorithm)
	processedCount := 0
	for len(queue) > 0 {
		// Dequeue node u
		u := queue[0]
		queue = queue[1:]
		processedCount++

		// For each dependent v of u
		for _, v := range dependents[u] {
			// Decrement in-degree of v
			inDegree[v]--

			// Update stage of v
			newStage := u.Stage + 1
			if newStage > v.Stage {
				v.Stage = newStage
			}

			// If in-degree[v] becomes 0, enqueue v
			if inDegree[v] == 0 {
				queue = append(queue, v)
			}
		}
	}

	// Check for cycles
	if processedCount != len(definitions) {
		// TODO: Identify the specific nodes involved in the cycle for a better error message
		return fmt.Errorf("cyclic dependency detected among definitions")
	}

	return nil
}

// DiscoverScopes traverses the AST starting from the root node and builds the initial scope hierarchy.
func DiscoverScopes(root *Root) *Context {
	// TODO: Implement scope discovery logic
	// 1. Create the root scope using NewScope("root", nil, false)
	// 2. Create a Context with the root scope.
	// 3. Implement an AST visitor (perhaps embedding NoopVisitor) to traverse the AST.
	// 4. Within the visitor methods (e.g., VisitModule, VisitFunctionStatement, VisitLetStatement):
	//    - Create new scopes for relevant nodes (modules, functions, blocks).
	//    - Create Definitions for declarations (functions, variables).
	//    - Track the current scope during traversal.
	//    - Populate Parent/Children links for scopes.
	//    - Populate the Definitions map in the current scope.
	//    - Store the corresponding AST Node in the Definition.
	//    - Initialise Definition Usages (this might be done in a separate pass or during CalculateStages).

	// Placeholder implementation: return a context with an empty root scope
	rootScope := NewScope("root", nil, false)
	return &Context{Root: rootScope}
}
