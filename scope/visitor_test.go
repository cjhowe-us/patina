package scope_test

import (
	"encoding/json"
	"strings"
	"testing"

	"github.com/cjhowedev/patina"
	"github.com/cjhowedev/patina/astutil"
	"github.com/cjhowedev/patina/scope"
	"github.com/gkampitakis/go-snaps/snaps"
)

// Simplified structures for snapshotting scope information
type SnapshotScope struct {
	Name        string                    `json:"name,omitempty"` // e.g., module name, function name
	Position    string                    `json:"position,omitempty"`
	Definitions map[string]string         `json:"definitions,omitempty"` // Map name to type (Var, Func, etc.)
	Children    map[string]*SnapshotScope `json:"children,omitempty"`    // Keyed by child scope name/identifier
}

type TestResult struct {
	Errors []string       `json:"errors,omitempty"`
	Scope  *SnapshotScope `json:"scope,omitempty"`
}

// Helper to convert the real scope tree to a snapshot-friendly version
func buildSnapshotScope(s *scope.Scope) *SnapshotScope {
	if s == nil {
		return nil
	}

	snapshot := &SnapshotScope{
		Position:    s.Position.String(),
		Definitions: make(map[string]string),
		Children:    make(map[string]*SnapshotScope),
	}

	// Try to derive a name from the associated node
	switch n := s.Node.(type) {
	case *patina.Module:
		snapshot.Name = "Module:" + n.Name
	case *patina.FunctionStatement:
		snapshot.Name = "Function:" + n.Name
	case *patina.Block:
		if n.Label != "" {
			snapshot.Name = "Block:" + n.Label
		} else {
			snapshot.Name = "Block@" + s.Position.String() // Use position for unnamed blocks
		}
	case *patina.Root:
		snapshot.Name = "Global"
	default:
		snapshot.Name = "Scope@" + s.Position.String()
	}

	for name, def := range s.Definitions {
		snapshot.Definitions[name] = def.Type.String()
	}

	for _, child := range s.Children {
		snapshotChild := buildSnapshotScope(child)
		if snapshotChild != nil {
			snapshot.Children[snapshotChild.Name] = snapshotChild
		}
	}
	if len(snapshot.Children) == 0 {
		snapshot.Children = nil // Omit empty children map in JSON
	}

	return snapshot
}

func TestScopeVisitor(t *testing.T) {
	testCases := map[string]string{
		"Basic Module": `
			module basic {
				let x = 1
				// Ensure no trailing comma in params
				fun add(a: i32, b: i32) {
					let sum = a + b
					return sum
				}
				let y = add(x, 2)
			}
		`,
		"Nested Blocks": `
			module nested {
				let outer_var = 1
				{
					let inner_var = 2
					{
						let innermost_var = 3
					}
				}
			}
		`,
		"Variable and Param Same Name": `
			module var_param_same {
				let x = 10
				// Ensure no trailing comma in params
				fun test(x: i32) {}
			}
		`,
		"Redefinition Error": `
			module redefine {
				let x = 1
				let x = 2 // Error
			}
		`,
		"Param Redefinition Error": `
			module param_redefine {
				fun bad(a: i32, a: i32) {} // Error
			}
		`,
		"Empty Input": "",
	}

	for name, source := range testCases {
		t.Run(name, func(t *testing.T) {
			ast, parseErr := patina.Parser.ParseString(name+".pat", source)
			// Allow parse errors for empty input case, but fail others
			if name == "Empty Input" {
				if parseErr == nil && source != "" {
					t.Fatal("Expected parse error for non-empty input that failed, but got nil")
				} else if source == "" && parseErr != nil && !strings.Contains(parseErr.Error(), "unexpected EOF") {
					// If source is empty, expect EOF error or nil error
					t.Fatalf("Expected EOF error for empty source, but got: %v", parseErr)
				}
				// For empty input or expected parse failure, snapshot just that.
				if ast == nil {
					snaps.MatchJSON(t, TestResult{Errors: []string{parseErr.Error()}})
					return
				}
			} else if parseErr != nil {
				t.Fatalf("Failed to parse source: %v", parseErr)
			}

			visitor := scope.NewScopeVisitor()
			_, walkErr := astutil.Walk(visitor, ast)
			if walkErr != nil {
				t.Fatalf("Walk returned an error: %v", walkErr)
			}

			result := TestResult{
				Errors: make([]string, len(visitor.Errors)),
				Scope:  buildSnapshotScope(visitor.RootScope),
			}
			for i, err := range visitor.Errors {
				result.Errors[i] = err.Error()
			}

			jsonData, marshalErr := json.MarshalIndent(result, "", "  ")
			if marshalErr != nil {
				t.Fatalf("Failed to marshal scope result: %v", marshalErr)
			}

			snaps.MatchJSON(t, string(jsonData))
		})
	}
}
