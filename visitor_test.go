package patina

import (
	"testing"

	"github.com/gkampitakis/go-snaps/snaps"
)

// TestVisitDispatch tests the basic Visit function dispatch (now using Unwrap helpers).
func TestVisitDispatch(t *testing.T) {
	t.Run("Visit function dispatches correctly", func(t *testing.T) {
		visitor := NewTestingVisitor()
		node := &Binding{Ident: "testIdentifier"} // Concrete type

		Visit(visitor, node)

		snaps.MatchJSON(t, visitor.Result())
	})

	t.Run("Visit function dispatches interface type correctly", func(t *testing.T) {
		visitor := NewTestingVisitor()
		var node Atom = &Binding{Ident: "testIdentifier"} // Interface type

		Visit(visitor, node)

		snaps.MatchJSON(t, visitor.Result())
	})

	t.Run("Visit function unwraps statement interface", func(t *testing.T) {
		visitor := NewTestingVisitor()
		var node Statement = &LetStatement{Name: "x"} // Statement interface

		Visit(visitor, node)

		snaps.MatchJSON(t, visitor.Result())
	})
}

// TestVisitorWithSnapshots uses table-driven tests and snapshotting
// to verify the RecursiveVisitor traversal and TestingVisitor data collection.
func TestVisitorWithSnapshots(t *testing.T) {
	tests := []struct {
		name string
		code string
	}{
		{
			name: "Simple binding",
			code: "let x = 1",
		},
		{
			name: "Function statement",
			code: "fun add(a: i32, b: i32): i32 { return a + b }",
		},
		{
			name: "If expression",
			code: `
				if (x > 10) {
					let large = x
				} else {
					let small = x
				}
			`,
		},
		{
			name: "Module with multiple statements",
			code: `module test {
				let x = 1
				fun main() {
					let y = x
				}
			}`,
		},
		{
			name: "String with interpolation",
			code: `let msg = "Hello, \(name)!"`,
		},
		{
			name: "Pointer type",
			code: "let ptr: *mut i32 = 0",
		},
		{
			name: "Complex nested expressions",
			code: "let x = (1 + (2 * (3 - 4)))",
		},
		{
			name: "Doc comment and annotations",
			code: `/* My doc comment */
			@tag.anno("hello") fun test() {}`,
		},
		{
			name: "Recursive factorial function", // Replaces TestRecursiveTraversal
			code: `
			module test {
				fun factorial(n: i32): i32 {
					if (n <= 1) {
						return 1
					} else {
						return n * factorial(n - 1)
					}
				}

				fun main() {
					let result = factorial(5)
				}
			}
			`,
		},
		{
			name: "Manual AST with nested blocks", // Replaces TestCustomVisitor somewhat
			code: `
			fun outer() {
				let a = 1
				{
					let b = 2
					{
						let c = 3
					}
				}
				let d = 4
			}
			`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Parse the code
			root, err := Parser.ParseString(tt.name, tt.code)
			if err != nil {
				t.Fatalf("Failed to parse code: %v\nCode:\n%s", err, tt.code)
			}

			// Visit the AST
			visitor := NewTestingVisitor()
			recursiveVisitor := &RecursiveVisitor{Delegate: visitor}
			Visit(recursiveVisitor, root)

			// Snapshot the results
			snaps.MatchJSON(t, visitor.Result())
		})
	}
}

// Remove old tests TestFakeVisitor, TestRecursiveTraversal, TestCustomVisitor
// as their functionality is covered by TestVisitorWithSnapshots

// Remove helper types like CountingVisitor and ASTVisitorFunc if they are no longer needed.
