package patina

import (
	"sort"
	"strings"
	"testing"

	"github.com/gkampitakis/go-snaps/snaps"
)

// sortResultsByName sorts a slice of maps by the "name" field for consistent snapshot comparison
func sortResultsByName(results []map[string]interface{}) {
	// Sort results by name
	sort.Slice(results, func(i, j int) bool {
		return results[i]["name"].(string) < results[j]["name"].(string)
	})

	// Sort usage names
	for _, result := range results {
		if usages, ok := result["usages"].([]string); ok && len(usages) > 0 {
			sort.Strings(usages)
		}
	}
}

func TestCalculateStages(t *testing.T) {
	tests := []struct {
		name           string
		setupScopeFunc func() *Context
	}{
		{
			name: "simple_no_dependencies",
			setupScopeFunc: func() *Context {
				// Create a scope with two definitions but no dependencies
				root := NewScope("root", nil, false)
				def1 := &Definition{
					Name:     "def1",
					IsStatic: true,
					Parent:   root,
					Usages:   nil,
				}
				def2 := &Definition{
					Name:     "def2",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}
				root.defs["def1"] = def1
				root.defs["def2"] = def2

				return &Context{Root: root}
			},
		},
		{
			name: "linear_chain",
			setupScopeFunc: func() *Context {
				// Create a scope with a chain of dependencies: a -> b -> c
				root := NewScope("root", nil, false)
				defA := &Definition{
					Name:     "a",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}
				defB := &Definition{
					Name:     "b",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}
				defC := &Definition{
					Name:     "c",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}

				// Set up usage relationships: a is used by b, b is used by c
				defA.Usages = []*Definition{defB}
				defB.Usages = []*Definition{defC}

				root.defs["a"] = defA
				root.defs["b"] = defB
				root.defs["c"] = defC

				return &Context{Root: root}
			},
		},
		{
			name: "branching_dependencies",
			setupScopeFunc: func() *Context {
				// Create a scope with branching dependencies: a -> b, a -> c
				root := NewScope("root", nil, false)
				defA := &Definition{
					Name:     "a",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}
				defB := &Definition{
					Name:     "b",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}
				defC := &Definition{
					Name:     "c",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}

				// Set up usage relationships: a is used by both b and c
				defA.Usages = []*Definition{defB, defC}

				root.defs["a"] = defA
				root.defs["b"] = defB
				root.defs["c"] = defC

				return &Context{Root: root}
			},
		},
		// Removed cyclic_dependencies test case - moved to TestCyclicDependencies
		{
			name: "static_and_nonstatic_mix",
			setupScopeFunc: func() *Context {
				// Create a scope with both static and non-static definitions
				root := NewScope("root", nil, false)
				defA := &Definition{
					Name:     "a",
					IsStatic: true, // Static
					Parent:   root,
					Usages:   nil,
				}
				defB := &Definition{
					Name:     "b",
					IsStatic: false, // Non-static
					Parent:   root,
					Usages:   nil,
				}
				defC := &Definition{
					Name:     "c",
					IsStatic: true, // Static using non-static
					Parent:   root,
					Usages:   nil,
				}

				// Set up usage relationships: Static c uses non-static b
				defB.Usages = []*Definition{defC}

				root.defs["a"] = defA
				root.defs["b"] = defB
				root.defs["c"] = defC

				return &Context{Root: root}
			},
		},
		{
			name: "nested_scopes",
			setupScopeFunc: func() *Context {
				// Create nested scopes with cross-scope references
				root := NewScope("root", nil, false)
				child := NewScope("child", root, false)
				root.children["child"] = child

				defA := &Definition{
					Name:     "a",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}
				defB := &Definition{
					Name:     "b",
					IsStatic: false,
					Parent:   child, // In child scope
					Usages:   nil,
				}

				// Set up usage relationship: a (in root) is used by b (in child)
				defA.Usages = []*Definition{defB}

				root.defs["a"] = defA
				child.defs["b"] = defB

				return &Context{Root: root}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Set up the test scope hierarchy
			ctx := tt.setupScopeFunc()

			// Calculate stages - handle error
			err := CalculateStages(ctx)
			if err != nil {
				t.Fatalf("CalculateStages returned unexpected error: %v", err)
			}

			// Create a snapshot-friendly representation of the results
			var results []map[string]interface{}
			allDefs := collectAllDefinitions(ctx.Root)
			for _, def := range allDefs {
				result := map[string]interface{}{
					"name":      def.Name,
					"is_static": def.IsStatic,
					"stage":     def.Stage,
					"usages":    getUsageNames(def),
				}
				results = append(results, result)
			}

			// Sort results for consistent snapshot comparison
			sortResultsByName(results)

			// Compare with snapshot using go-snaps
			snapsConfig := snaps.WithConfig(jsonSnapshotConfig)
			snapsConfig.MatchJSON(t, results)
		})
	}
}

// TestCyclicDependencies checks that cyclic dependencies are detected and reported correctly
func TestCyclicDependencies(t *testing.T) {
	tests := []struct {
		name              string
		setupScopeFunc    func() *Context
		expectedErrorText string
	}{
		{
			name: "simple_cycle",
			setupScopeFunc: func() *Context {
				// Create a scope with cyclic dependencies: a -> b -> c -> a
				root := NewScope("root", nil, false)
				defA := &Definition{
					Name:     "a",
					IsStatic: true,
					Parent:   root,
					Usages:   nil,
				}
				defB := &Definition{
					Name:     "b",
					IsStatic: true,
					Parent:   root,
					Usages:   nil,
				}
				defC := &Definition{
					Name:     "c",
					IsStatic: true,
					Parent:   root,
					Usages:   nil,
				}

				// Set up cyclic usage relationships
				defA.Usages = []*Definition{defB}
				defB.Usages = []*Definition{defC}
				defC.Usages = []*Definition{defA}

				root.defs["a"] = defA
				root.defs["b"] = defB
				root.defs["c"] = defC

				return &Context{Root: root}
			},
			expectedErrorText: "cyclic dependency detected: a -> b -> c -> a",
		},
		{
			name: "self_cycle",
			setupScopeFunc: func() *Context {
				// Create a scope with a self-referential cycle: a -> a
				root := NewScope("root", nil, false)
				defA := &Definition{
					Name:     "a",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}

				// Self-referential usage
				defA.Usages = []*Definition{defA}

				root.defs["a"] = defA

				return &Context{Root: root}
			},
			expectedErrorText: "cyclic dependency detected: a -> a",
		},
		{
			name: "complex_cycle",
			setupScopeFunc: func() *Context {
				// Create a more complex dependency cycle: a -> b -> c -> d -> b
				root := NewScope("root", nil, false)
				defA := &Definition{
					Name:     "a",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}
				defB := &Definition{
					Name:     "b",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}
				defC := &Definition{
					Name:     "c",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}
				defD := &Definition{
					Name:     "d",
					IsStatic: false,
					Parent:   root,
					Usages:   nil,
				}

				// Set up complex cyclic dependencies
				defA.Usages = []*Definition{defB}
				defB.Usages = []*Definition{defC}
				defC.Usages = []*Definition{defD}
				defD.Usages = []*Definition{defB} // Cycle back to B

				root.defs["a"] = defA
				root.defs["b"] = defB
				root.defs["c"] = defC
				root.defs["d"] = defD

				return &Context{Root: root}
			},
			expectedErrorText: "cyclic dependency detected:",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Set up the test scope hierarchy
			ctx := tt.setupScopeFunc()

			// Calculate stages - should return error for cyclic dependencies
			err := CalculateStages(ctx)

			// Verify error is returned
			if err == nil {
				t.Fatalf("Expected error for cyclic dependency, but got nil")
			}

			// Check error message
			if !strings.Contains(err.Error(), tt.expectedErrorText) {
				t.Errorf("Error message didn't match expected text.\nGot: %s\nWanted to contain: %s",
					err.Error(), tt.expectedErrorText)
			}
		})
	}
}

// getUsageNames returns the names of all definitions that this definition depends on (dependencies)
func getUsageNames(def *Definition) []string {
	var names []string
	for _, usage := range def.Usages {
		names = append(names, usage.Name)
	}
	return names
}

func TestStageIntegration(t *testing.T) {
	tests := []struct {
		name       string
		patinaCode string
	}{
		{
			name: "simple_variables",
			patinaCode: `
module test {
	let a = 1
	let b = a
	let c = b

	fun main() {
		let d = c
	}
}
`,
		},
		{
			name: "static_variables",
			patinaCode: `
module test {
	static let a = 1
	let b = a
	static let c = b
}
`,
		},
		{
			name: "function_calls",
			patinaCode: `
module test {
	fun a() {}
	fun b() { a() }
	fun c() { b() }

	fun main() {
		c()
	}
}
`,
		},
		{
			name: "mixed_dependencies",
			patinaCode: `
module test {
	let x = 1
	fun getX() { return x }
	let y = getX()

	static fun staticFn() { return 42 }
	let z = staticFn()
}
`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Parse the code - convert string to io.Reader
			root, err := Parser.Parse("test.patina", strings.NewReader(tt.patinaCode))
			if err != nil {
				t.Fatalf("Failed to parse Patina code: %v", err)
			}

			// Discover scopes and calculate stages
			ctx := DiscoverScopes(root)

			// Calculate stages - handle error
			err = CalculateStages(ctx)
			if err != nil {
				t.Fatalf("CalculateStages returned unexpected error: %v", err)
			}

			// Create a snapshot-friendly representation of the results
			var results []map[string]interface{}
			allDefs := collectAllDefinitions(ctx.Root)
			for _, def := range allDefs {
				result := map[string]interface{}{
					"name":      def.Name,
					"is_static": def.IsStatic,
					"stage":     def.Stage,
					"usages":    getUsageNames(def),
				}
				results = append(results, result)
			}

			// Sort results for consistent snapshot comparison
			sortResultsByName(results)

			// Compare with snapshot using go-snaps
			snapsConfig := snaps.WithConfig(jsonSnapshotConfig)
			snapsConfig.MatchJSON(t, results)
		})
	}
}

func TestCyclicDependenciesInPatinaCode(t *testing.T) {
	tests := []struct {
		name              string
		patinaCode        string
		expectedErrorText string
	}{
		{
			name: "simple_cycle",
			patinaCode: `
module test {
	let a = b
	let b = c
	let c = a
}
`,
			expectedErrorText: "cyclic dependency detected:",
		},
		{
			name: "self_referencing",
			patinaCode: `
module test {
	let a = a
}
`,
			expectedErrorText: "cyclic dependency detected: a -> a",
		},
		{
			name: "function_cycle",
			patinaCode: `
module test {
	fun a() { b() }
	fun b() { c() }
	fun c() { a() }
}
`,
			expectedErrorText: "cyclic dependency detected:",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Parse the code
			root, err := Parser.Parse("test.patina", strings.NewReader(tt.patinaCode))
			if err != nil {
				t.Fatalf("Failed to parse Patina code: %v", err)
			}

			// Discover scopes
			ctx := DiscoverScopes(root)

			// Calculate stages - should return error for cyclic dependencies
			err = CalculateStages(ctx)

			// Verify error is returned
			if err == nil {
				t.Fatalf("Expected error for cyclic dependency, but got nil")
			}

			// Check error message
			if !strings.Contains(err.Error(), tt.expectedErrorText) {
				t.Errorf("Error message didn't match expected text.\nGot: %s\nWanted to contain: %s",
					err.Error(), tt.expectedErrorText)
			}
		})
	}
}
