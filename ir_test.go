package patina

import (
	"testing"

	"github.com/gkampitakis/go-snaps/snaps"
)

func TestIRConverter(t *testing.T) {
	type converterTestCase struct {
		name       string
		src        string
		stageCount int
	}

	tests := []converterTestCase{
		{
			name: "Module with stage separation",
			src: `module test {
				static let x = *mut i32
				let y: (static x) = 42
			}`,
			stageCount: 1,
		},
		{
			name:       "Simple binding across stages",
			src:        "static let x = 1\nlet y = x", // x is compile-time, y is runtime
			stageCount: 1,
		},
		{
			name: "Multi-stage compilation",
			src: `
			// Stage 0 (compile time)
			static let compileVal = 42

			// Stage 1 (intermediate)
			static let midVal = compileVal + 1

			// Runtime stage
			let runtimeVal = midVal * 2
			`,
			stageCount: 2, // 2 compile stages + runtime
		},
		{
			name: "Static function generating code",
			src: `
			static fun generateCode() {
				// This function runs at compile time
				let runtime_code = static {
					// This is a quoted expression (runtime code)
					fun main() {
						print("Hello from runtime!")
					}
				}
				return runtime_code
			}

			// Call the compile-time function
			static generateCode()
			`,
			stageCount: 1,
		},
		{
			name: "Type expressions across stages",
			src: `
			static let IntType = i64
			let x: (IntType) = 42
			`,
			stageCount: 1,
		},
		{
			name:       "Empty input",
			src:        "",
			stageCount: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Parse the source
			root, err := Parser.ParseString(tt.name, tt.src)
			if err != nil {
				t.Fatalf("Unexpected error parsing source: %v\n%s", err, tt.src)
			}

			// Convert to IR
			converter := NewIRConverter(tt.stageCount)
			ir, err := converter.Convert(root)
			if err != nil {
				t.Fatalf("Unexpected error converting to IR: %v\n%s", err, tt.src)
			}

			// Verify structure
			if len(ir.Stages) != tt.stageCount+1 { // +1 for runtime stage
				t.Fatalf("Expected %d stages, got %d", tt.stageCount+1, len(ir.Stages))
			}

			// Ensure runtime stage is the last one
			lastStage := ir.Stages[len(ir.Stages)-1]
			if lastStage.Level != Runtime {
				t.Fatalf("Expected last stage to be Runtime, got %v", lastStage.Level)
			}

			// Match the snapshot
			snaps.MatchYAML(t, ir)
		})
	}
}

func TestIRConverterErrors(t *testing.T) {
	type converterErrorTestCase struct {
		name        string
		src         string
		stageCount  int
		expectedErr string
	}

	tests := []converterErrorTestCase{
		{
			name:        "Nil root",
			src:         "",
			stageCount:  1,
			expectedErr: "cannot convert nil root",
		},
		{
			name:        "Stage reference exceeding configured stages",
			src:         "@stage.2 let x = 42", // Referencing stage 2 when only 1 is configured
			stageCount:  1,
			expectedErr: "statement requires stage 2 but only 1 compile stages are configured",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var root *Root
			var err error

			if tt.src != "" {
				// Parse the source
				root, err = Parser.ParseString(tt.name, tt.src)
				if err != nil {
					t.Skipf("Skipping due to parse error (not testing parser): %v", err)
					return
				}
			}

			// Convert to IR
			converter := NewIRConverter(tt.stageCount)
			_, err = converter.Convert(root)

			// Check for expected error
			if err == nil {
				t.Fatalf("Expected error but got none")
			}

			// Match error snapshot or check error message
			if tt.expectedErr != "" && err.Error() != tt.expectedErr {
				t.Fatalf("Expected error '%s', got '%s'", tt.expectedErr, err.Error())
			} else {
				snaps.MatchSnapshot(t, err.Error())
			}
		})
	}
}

// Additional tests to specifically validate staging behavior
func TestStaging(t *testing.T) {
	type stagingTestCase struct {
		name       string
		src        string
		stageCount int
		test       func(t *testing.T, ir *IR)
	}

	tests := []stagingTestCase{
		{
			name: "Static expressions are always compile-time",
			src: `let x = static {
				let y = 42
				y + 1
			}`,
			stageCount: 1,
			test: func(t *testing.T, ir *IR) {
				// Check that x is in the runtime stage
				runtimeStage := ir.Stages[ir.stageCount()]
				if len(runtimeStage.Root.Statements) == 0 {
					t.Fatal("Expected statement in runtime stage")
				}

				// But there should be cross-stage values from compile time
				if len(runtimeStage.CrossStageValues) == 0 {
					t.Fatal("Expected cross-stage values in runtime stage")
				}

				// And the expression itself should be in the compile-time stage
				compileStage := ir.Stages[0]
				hasStageExpr := false

				// This is a simplistic check - in a real test, we'd have more specific assertions
				for _, item := range compileStage.GeneratedFragments {
					if item.Source != nil {
						hasStageExpr = true
						break
					}
				}

				if !hasStageExpr {
					t.Fatal("Did not find expected stage expression in compile stage")
				}
			},
		},
		{
			name:       "Type expressions generate quotes",
			src:        `let x = i32`,
			stageCount: 1,
			test: func(t *testing.T, ir *IR) {
				// Check that the let binding is in the runtime stage
				runtimeStage := ir.Stages[len(ir.Stages)-1]
				if len(runtimeStage.Root.Statements) == 0 {
					t.Fatal("Expected statement in runtime stage")
				}

				// There should be a generated fragment from the type expression
				if len(runtimeStage.GeneratedFragments) == 0 {
					t.Fatal("Expected generated fragments in runtime stage")
				}
			},
		},
		{
			name: "Multi-stage sharing of values",
			src: `
			static let x = 1      // stage 0
			@stage.1 let y = x + 1  // stage 1
			let z = y + 1         // runtime
			`,
			stageCount: 2, // 2 compile stages + runtime
			test: func(t *testing.T, ir *IR) {
				// Should have 3 stages (2 compile + runtime)
				if len(ir.Stages) != 3 {
					t.Fatalf("Expected 3 stages, got %d", len(ir.Stages))
				}

				// Stage 0 should have the x binding
				stage0 := ir.Stages[0]
				if len(stage0.Root.Statements) != 1 {
					t.Fatalf("Expected 1 statement in stage 0, got %d", len(stage0.Root.Statements))
				}

				// Stage 1 should have the y binding and cross-stage values from stage 0
				stage1 := ir.Stages[1]
				if len(stage1.Root.Statements) != 1 {
					t.Fatalf("Expected 1 statement in stage 1, got %d", len(stage1.Root.Statements))
				}
				if len(stage1.CrossStageValues) == 0 {
					t.Fatal("Expected cross-stage values in stage 1")
				}

				// Runtime should have the z binding and cross-stage values from stage 1
				runtimeStage := ir.Stages[2]
				if len(runtimeStage.Root.Statements) != 1 {
					t.Fatalf("Expected 1 statement in runtime stage, got %d", len(runtimeStage.Root.Statements))
				}
				if len(runtimeStage.CrossStageValues) == 0 {
					t.Fatal("Expected cross-stage values in runtime stage")
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Parse the source
			root, err := Parser.ParseString(tt.name, tt.src)
			if err != nil {
				t.Fatalf("Unexpected error parsing source: %v\n%s", err, tt.src)
			}

			// Convert to IR
			converter := NewIRConverter(tt.stageCount)
			ir, err := converter.Convert(root)
			if err != nil {
				t.Fatalf("Unexpected error converting to IR: %v\n%s", err, tt.src)
			}

			// Run the test-specific assertions
			tt.test(t, ir)
		})
	}
}

// Helper for tests
func (ir *IR) stageCount() int {
	return len(ir.Stages) - 1 // -1 because the last stage is always runtime
}
