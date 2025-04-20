package dependency_test

import (
	"encoding/json"
	"sort"
	"testing"

	"github.com/cjhowedev/patina"
	"github.com/cjhowedev/patina/astutil"
	"github.com/cjhowedev/patina/dependency"
	"github.com/cjhowedev/patina/scope"
	"github.com/gkampitakis/go-snaps/snaps"
)

// Simplified structures for snapshotting dependency information
type SnapshotDependency struct {
	From string   `json:"from"` // Definition name + position
	To   []string `json:"to"`   // List of dependency names + positions
}

type TestResult struct {
	Errors       []string             `json:"errors,omitempty"`
	Dependencies []SnapshotDependency `json:"dependencies,omitempty"`
}

// Helper to convert definition to string key for map/sorting
func defKey(def *scope.Definition) string {
	if def == nil {
		return "<nil>"
	}
	return def.Name + "@" + def.Position.String()
}

func TestDependencyVisitor(t *testing.T) {
	testCases := map[string]string{
		"Simple Let": `
			module simple {
				let x = 1
				let y = x // y depends on x
			}
		`,
		"Function Call": `
			module funccall {
				fun ident(a: i32): i32 { return a }
				let result = ident(5) // result depends on ident
			}
		`,
		"Transitive Dependency": `
			module transitive {
				let a = 1
				let b = a
				let c = b // c depends on b, b depends on a
			}
		`,
		"Type Dependency": `
			module types {
				// Changed Foo to i32 as type defs are not supported yet
				let x: i32
			}
		`,
		"Function Signature Dependency": `
			module sigdep {
				// Changed T1, T2, T3 to i32
				fun process(p1: i32, p2: i32): i32 {
					let local: i32 = p1
					return local
				}
			}
		`,
		"Undefined Identifier": `
			module undef {
				let x = y // Error: y is not defined
			}
		`,
		"Self Dependency (Ignored)": `
			module selfdep {
				let x = x // Should not create a dependency cycle in the graph
			}
		`,
		"Mutual Dependency": `
			module mutual {
				// Direct mutual dependencies usually require function declarations + calls
				fun ping(n: i32) { if (n > 0) { pong(n - 1) } }
				fun pong(n: i32) { if (n > 0) { ping(n - 1) } }
				let start = ping(5) // start depends on ping
				// ping depends on pong, pong depends on ping
			}
		`,
	}

	for name, source := range testCases {
		t.Run(name, func(t *testing.T) {
			ast, parseErr := patina.Parser.ParseString(name+".pat", source)
			if parseErr != nil {
				t.Fatalf("Failed to parse source: %v", parseErr)
			}

			// 1. Run Scope Visitor
			scopeVisitor := scope.NewScopeVisitor()
			_, scopeWalkErr := astutil.Walk(scopeVisitor, ast)
			if scopeWalkErr != nil {
				t.Fatalf("Scope Walk returned an error: %v", scopeWalkErr)
			}
			if len(scopeVisitor.Errors) > 0 {
				// If scope analysis failed, don't proceed to dependency analysis
				t.Logf("Scope analysis errors found, skipping dependency check:")
				for _, err := range scopeVisitor.Errors {
					t.Logf("- %v", err)
				}
				// Snapshot scope errors anyway
				errors := make([]string, len(scopeVisitor.Errors))
				for i, err := range scopeVisitor.Errors {
					errors[i] = err.Error()
				}
				snaps.MatchJSON(t, TestResult{Errors: errors})
				return
			}

			// 2. Build Scope Map
			scopeMap := dependency.BuildScopeMap(scopeVisitor.RootScope)

			// 3. Run Dependency Visitor
			depVisitor := dependency.NewDependencyVisitor(scopeVisitor.RootScope, scopeMap)
			_, depWalkErr := astutil.Walk(depVisitor, ast)
			if depWalkErr != nil {
				t.Fatalf("Dependency Walk returned an error: %v", depWalkErr)
			}

			// 4. Prepare result for snapshotting
			result := TestResult{
				Errors:       make([]string, len(depVisitor.Errors)),
				Dependencies: []SnapshotDependency{},
			}
			for i, err := range depVisitor.Errors {
				result.Errors[i] = err.Error()
			}

			// Convert graph to a slice and sort for stable snapshots
			var sortedKeys []*scope.Definition
			for def := range depVisitor.Graph {
				sortedKeys = append(sortedKeys, def)
			}
			sort.Slice(sortedKeys, func(i, j int) bool {
				return defKey(sortedKeys[i]) < defKey(sortedKeys[j])
			})

			for _, def := range sortedKeys {
				deps := depVisitor.Graph[def]
				// Sort dependencies for stable snapshots
				sort.Slice(deps, func(i, j int) bool {
					return defKey(deps[i]) < defKey(deps[j])
				})
				depStrings := make([]string, len(deps))
				for i, d := range deps {
					depStrings[i] = defKey(d)
				}
				result.Dependencies = append(result.Dependencies, SnapshotDependency{
					From: defKey(def),
					To:   depStrings,
				})
			}

			jsonData, marshalErr := json.MarshalIndent(result, "", "  ")
			if marshalErr != nil {
				t.Fatalf("Failed to marshal dependency result: %v", marshalErr)
			}

			snaps.MatchJSON(t, string(jsonData))
		})
	}
}
