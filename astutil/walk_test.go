package astutil_test

import (
	"testing"

	"github.com/cjhowedev/patina"
	"github.com/cjhowedev/patina/astutil"
	"github.com/gkampitakis/go-snaps/snaps"
)

func TestWalk(t *testing.T) {
	testCases := map[string]string{
		"Minimal Module": `
			module test {
				/* Doc Comment */
				// Line Comment (elided)
				@Anno()
				@AnnoWithArgs(1, "hello")
				fun main() {
					let x = 1
				}
			}
		`,
		"Complex Expression": `
			module expr {
				fun calc(a: i32, b: i32): i32 {
					let complex = if (a > b) {
						a + 1
					} else if (a < b) {
						b - 1
					} else {
						0 // Equal case
					}
					return complex * 2
				}
			}
		`,
		"Types and Pointers": `
			module types {}
		`,
		"String Interpolation": `
			module interp {
				fun greet(name: String) {
					let message = "Hello, \(name)!"
				}
			}
		`,
		"Empty Module": `module empty {}`,
		"Annotations Only": `
			module annos {
				@GlobalAnno
				fun annotated() {}
			}
		`,
		"Let without value": `
			module letnoval {
				let x: i32
			}
		`,
		"Return with label": `
			module retlabel {
				fun find(): i32 :loop {
					return :loop 1
				}
			}
		`,
		"Break with value": `
			module breakval {
				fun search(): i32 :outer {
					let i = 0
					:inner {
						if (i > 10) {
							break :outer i
						}
						i = i + 1
					}
					return -1 // Should not be reached if break happens
				}
			}
		`,
		"Case Expression": `
			module cases {
				fun check(v: i32) {
					case {
						if (v > 0) { 1 }
						if (v < 0) { -1 }
						else { 0 }
					}
				}
			}
		`,
		"Lambda Expression": `
			module lambdas {
				fun apply(f: FuncTypePlaceholder, x: i32): i32 {
					return x
				}
				let incrementer = fun(n: i32): i32 { n + 1 }
				let result = 5
			}
		`,
		"Static Let": `
			module statics {
				static let PI = 3
			}
		`,
	}

	for name, source := range testCases {
		t.Run(name, func(t *testing.T) {
			ast, err := patina.Parser.ParseString(name+".pat", source)
			if err != nil {
				t.Fatalf("Failed to parse source: %v", err)
			}

			visitor := astutil.SnapshotVisitor{}
			snapshotResult, walkErr := astutil.Walk(visitor, ast)
			if walkErr != nil {
				t.Fatalf("Walk returned an error: %v", walkErr)
			}

			// Marshal the snapshot result for comparison
			jsonData, marshalErr := astutil.MarshalSnapshot(snapshotResult)
			if marshalErr != nil {
				t.Fatalf("Failed to marshal snapshot result: %v", marshalErr)
			}

			snaps.MatchJSON(t, string(jsonData))
		})
	}
}
